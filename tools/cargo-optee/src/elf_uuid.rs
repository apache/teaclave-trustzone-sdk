// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! UUID extraction from unsigned OP-TEE 4.10.0 TA ELF images and plugins.
//!
//! TA: `ldelf` resolves `ta_head` from dynsym; UUID is the first 16 bytes of
//! `struct ta_head` (`user_ta_header.h`).
//! Plugin: `tee-supplicant` uses `dlsym("plugin_method")`; UUID follows the
//! leading `name` pointer in `struct plugin_method` (`tee_plugin_method.h`;
//! pointer width from the ELF class). Both symbols must be objects large
//! enough to cover the UUID field.

use anyhow::{Context, Result, bail, ensure};
use goblin::elf::header::{ELFCLASS32, ELFCLASS64, EM_AARCH64, EM_ARM, ET_DYN};
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::{Elf, header, sym};
use std::path::Path;
use uuid::Uuid;

const TEE_UUID_SIZE: usize = 16;

/// Read the TA UUID from an unsigned TA ELF.
pub fn read_ta_uuid(path: &Path) -> Result<Uuid> {
    let data =
        std::fs::read(path).with_context(|| format!("{}: failed to read ELF", path.display()))?;
    read_ta_uuid_from_bytes(path, &data)
}

/// Read the plugin UUID from an unsigned plugin shared library.
pub fn read_plugin_uuid(path: &Path) -> Result<Uuid> {
    let data =
        std::fs::read(path).with_context(|| format!("{}: failed to read ELF", path.display()))?;
    read_plugin_uuid_from_bytes(path, &data)
}

fn read_ta_uuid_from_bytes(path: &Path, data: &[u8]) -> Result<Uuid> {
    let elf = parse_optee_elf(path, data)?;
    let sym = exported_symbol(&elf, "ta_head").ok_or_else(|| {
        if has_named_sym(&elf, "ta_head") {
            anyhow::anyhow!(
                "{}: ta_head exists in the regular symbol table but is not dynamically exported; ldelf resolves it from dynsym",
                path.display()
            )
        } else {
            anyhow::anyhow!(
                "{}: dynamically exported symbol ta_head not found",
                path.display()
            )
        }
    })?;
    ensure_object(path, "ta_head", &sym, TEE_UUID_SIZE as u64)?;
    tee_uuid(bytes_at_va(path, &elf, data, sym.st_value, TEE_UUID_SIZE)?)
}

fn read_plugin_uuid_from_bytes(path: &Path, data: &[u8]) -> Result<Uuid> {
    let elf = parse_optee_elf(path, data)?;
    let sym = exported_symbol(&elf, "plugin_method").ok_or_else(|| {
        if has_named_sym(&elf, "plugin_method") {
            anyhow::anyhow!(
                "{}: plugin_method exists in the regular symbol table but is not dynamically exported; tee-supplicant resolves it with dlsym and a non-exported object cannot be used",
                path.display()
            )
        } else {
            anyhow::anyhow!(
                "{}: dynamically exported symbol plugin_method not found",
                path.display()
            )
        }
    })?;
    let uuid_off = pointer_width(&elf);
    ensure_object(path, "plugin_method", &sym, uuid_off + TEE_UUID_SIZE as u64)?;
    let va = sym.st_value.checked_add(uuid_off).with_context(|| {
        format!(
            "{}: plugin_method address {:#x} overflowed when skipping the name pointer",
            path.display(),
            sym.st_value
        )
    })?;
    tee_uuid(bytes_at_va(path, &elf, data, va, TEE_UUID_SIZE)?)
}

fn parse_optee_elf<'a>(path: &Path, data: &'a [u8]) -> Result<Elf<'a>> {
    ensure!(
        data.starts_with(b"\x7fELF"),
        "{}: not an ELF file: missing ELF magic",
        path.display()
    );
    let elf = Elf::parse(data).with_context(|| {
        format!(
            "{}: failed to parse ELF (truncated or malformed)",
            path.display()
        )
    })?;
    ensure!(
        elf.little_endian,
        "{}: unsupported ELF endianness (OP-TEE ARM/AArch64 TAs and plugins are little-endian)",
        path.display()
    );
    ensure!(
        elf.header.e_type == ET_DYN,
        "{}: unsupported ELF type {} (OP-TEE 4.10 requires ET_DYN for TAs and plugins)",
        path.display(),
        elf.header.e_type
    );
    match (
        elf.header.e_machine,
        elf.header.e_ident[header::EI_CLASS],
        elf.is_64,
    ) {
        (EM_AARCH64, ELFCLASS64, true) | (EM_ARM, ELFCLASS32, false) => Ok(elf),
        (machine, class, is_64) => bail!(
            "{}: unsupported ELF target machine={} class={} is_64={} (supported: AArch64 little-endian 64-bit, ARM little-endian 32-bit)",
            path.display(),
            machine,
            class,
            is_64
        ),
    }
}

fn pointer_width(elf: &Elf<'_>) -> u64 {
    if elf.is_64 { 8 } else { 4 }
}

fn exported_symbol(elf: &Elf<'_>, name: &str) -> Option<goblin::elf::Sym> {
    elf.dynsyms.iter().find(|sym| {
        elf.dynstrtab.get_at(sym.st_name) == Some(name)
            && sym.st_shndx != 0
            && matches!(sym.st_bind(), sym::STB_GLOBAL | sym::STB_WEAK)
    })
}

/// `ta_head` and `plugin_method` are C structs (`struct ta_head`,
/// `struct plugin_method`). Reject functions and undersized objects so a
/// shifted layout fails instead of returning a neighboring field as a UUID.
/// `st_size == 0` means the size is unknown.
fn ensure_object(path: &Path, name: &str, sym: &goblin::elf::Sym, min_size: u64) -> Result<()> {
    match sym.st_type() {
        sym::STT_OBJECT | sym::STT_NOTYPE => {}
        t => bail!(
            "{}: {name} has ELF symbol type {t} (expected STT_OBJECT)",
            path.display()
        ),
    }
    if sym.st_size != 0 {
        ensure!(
            sym.st_size >= min_size,
            "{}: {name} is {} bytes, need at least {min_size} to read the UUID",
            path.display(),
            sym.st_size
        );
    }
    Ok(())
}

fn has_named_sym(elf: &Elf<'_>, name: &str) -> bool {
    elf.syms
        .iter()
        .any(|sym| elf.strtab.get_at(sym.st_name) == Some(name) && sym.st_shndx != 0)
}

fn bytes_at_va<'a>(
    path: &Path,
    elf: &Elf<'_>,
    data: &'a [u8],
    va: u64,
    size: usize,
) -> Result<&'a [u8]> {
    let size_u64 = u64::try_from(size).map_err(|_| {
        anyhow::anyhow!(
            "{}: malformed ELF: requested {} bytes at {:#x}",
            path.display(),
            size,
            va
        )
    })?;
    let end = va.checked_add(size_u64).with_context(|| {
        format!(
            "{}: malformed ELF: virtual address overflow at {:#x}",
            path.display(),
            va
        )
    })?;
    for ph in elf.program_headers.iter().filter(|ph| ph.p_type == PT_LOAD) {
        let load_end = ph.p_vaddr.checked_add(ph.p_memsz).with_context(|| {
            format!(
                "{}: malformed ELF: PT_LOAD p_vaddr {:#x} + p_memsz {:#x} overflow",
                path.display(),
                ph.p_vaddr,
                ph.p_memsz
            )
        })?;
        if va < ph.p_vaddr || end > load_end {
            continue;
        }
        let file_end = ph.p_vaddr.checked_add(ph.p_filesz).with_context(|| {
            format!(
                "{}: malformed ELF: PT_LOAD p_vaddr {:#x} + p_filesz {:#x} overflow",
                path.display(),
                ph.p_vaddr,
                ph.p_filesz
            )
        })?;
        ensure!(
            end <= file_end,
            "{}: virtual address {:#x} is in BSS of a PT_LOAD segment (not file-backed); UUID cannot be read from the file",
            path.display(),
            va
        );
        let delta = va - ph.p_vaddr;
        let off = ph.p_offset.checked_add(delta).with_context(|| {
            format!(
                "{}: malformed ELF: PT_LOAD file offset overflow at {:#x}",
                path.display(),
                va
            )
        })?;
        let off = usize::try_from(off).map_err(|_| {
            anyhow::anyhow!(
                "{}: malformed ELF: file offset {:#x} does not fit in usize",
                path.display(),
                off
            )
        })?;
        let end_off = off.checked_add(size).with_context(|| {
            format!(
                "{}: malformed ELF: file range overflow at offset {}",
                path.display(),
                off
            )
        })?;
        return data.get(off..end_off).with_context(|| {
            format!(
                "{}: offset {} size {} exceeds file length {}",
                path.display(),
                off,
                size,
                data.len()
            )
        });
    }
    bail!(
        "{}: virtual address {:#x} is not in any PT_LOAD segment",
        path.display(),
        va
    )
}

fn tee_uuid(bytes: &[u8]) -> Result<Uuid> {
    let b: [u8; 16] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("TEE_UUID must be 16 bytes, got {}", bytes.len()))?;
    Ok(Uuid::from_bytes_le(b))
}
