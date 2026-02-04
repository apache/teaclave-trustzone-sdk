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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use crate::Error;

/// The type of the linker, there are difference when using gcc/cc or ld/lld as  
/// linker, For example, `--sort-section=alignment` parameter changes to  
/// `-Wl,--sort-section=alignment` when using gcc as linker.
///
/// Cc: gcc, cc, etc.
/// Ld: ld, lld, ld.bfd, ld.gold, etc.
#[derive(Debug, Clone)]
pub enum LinkerType {
    Cc,
    Ld,
}

/// Linker of ta, use it to handle all linking stuff.
///
/// Use only if you just want to handle the linking stuff, and use a  
/// hand-written user_ta_header.rs, or you should use Builder instead.
/// Usage:
///
/// ```no_run
/// use optee_utee_build::Linker;
/// use std::env;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// let out_dir = env::var("OUT_DIR")?;
/// Linker::auto().link_all(out_dir)?;
/// # Ok(())
/// # }
/// ```
///
/// We detect the type of the linker automatically, you can set it manually if
/// you met some problems with it.
/// ```no_run
/// use optee_utee_build::{Linker, LinkerType};
/// use std::env;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// let out_dir = env::var("OUT_DIR")?;
/// Linker::new(LinkerType::Cc).link_all(out_dir)?;
/// # Ok(())
/// # }
/// ```
///
pub struct Linker {
    linker_type: LinkerType,
    ftrace_buf_size: Option<usize>,
}

impl Linker {
    /// Construct a Linker by manually specific the type of linker, you may use
    /// `auto`, it would detect current linker automatically.
    pub fn new(linker_type: LinkerType) -> Self {
        Self {
            linker_type,
            ftrace_buf_size: None,
        }
    }
    /// Construct a Linker by auto detect the type of linker, try `new` function
    ///  if our detection mismatch.
    pub fn auto() -> Self {
        Self::new(Self::auto_detect_linker_type())
    }
    /// Set the ftrace buffer size
    pub fn with_ftrace_buf_size(mut self, ftrace_buf_size: usize) -> Self {
        self.ftrace_buf_size = Some(ftrace_buf_size);
        self
    }
    /// Handle all the linking stuff.
    ///
    /// param out_dir is used for putting some generated files that linker would
    ///  use.
    pub fn link_all<P: Into<PathBuf>>(self, out_dir: P) -> Result<(), Error> {
        const ENV_TA_DEV_KIT_DIR: &str = "TA_DEV_KIT_DIR";
        println!("cargo:rerun-if-env-changed={}", ENV_TA_DEV_KIT_DIR);
        let ta_dev_kit_dir = PathBuf::from(std::env::var(ENV_TA_DEV_KIT_DIR)?);
        let out_dir: PathBuf = out_dir.into();

        self.write_and_set_linker_script(out_dir.clone(), ta_dev_kit_dir.clone())?;

        let search_path = ta_dev_kit_dir.join("lib");
        println!("cargo:rustc-link-search={}", search_path.display());
        println!("cargo:rustc-link-lib=static=utee");
        println!("cargo:rustc-link-lib=static=utils");
        println!("cargo:rustc-link-arg=-e__ta_entry");
        println!("cargo:rustc-link-arg=-pie");
        println!("cargo:rustc-link-arg=-Os");
        match self.linker_type {
            LinkerType::Cc => println!("cargo:rustc-link-arg=-Wl,--sort-section=alignment"),
            LinkerType::Ld => println!("cargo:rustc-link-arg=--sort-section=alignment"),
        };
        let mut dyn_list = File::create(out_dir.join("dyn_list"))?;
        writeln!(
            dyn_list,
            "{{ __elf_phdr_info; trace_ext_prefix; trace_level; ta_head; }};"
        )?;
        match self.linker_type {
            LinkerType::Cc => println!("cargo:rustc-link-arg=-Wl,--dynamic-list=dyn_list"),
            LinkerType::Ld => println!("cargo:rustc-link-arg=--dynamic-list=dyn_list"),
        }

        Ok(())
    }
}

impl Linker {
    // generate a link script file for cc/ld, and link to it
    fn write_and_set_linker_script(
        &self,
        out_dir: PathBuf,
        ta_dev_kit_dir: PathBuf,
    ) -> Result<(), Error> {
        // cargo passes TARGET as env to the build scripts
        const ENV_TARGET: &str = "TARGET";
        println!("cargo:rerun-if-env-changed={}", ENV_TARGET);
        match env::var(ENV_TARGET) {
            Ok(ref v) if v == "arm-unknown-linux-gnueabihf" || v == "arm-unknown-optee" => {
                match self.linker_type {
                    LinkerType::Cc => println!("cargo:rustc-link-arg=-Wl,--no-warn-mismatch"),
                    LinkerType::Ld => println!("cargo:rustc-link-arg=--no-warn-mismatch"),
                };
            }
            _ => {}
        };

        let link_script_dest = out_dir.join("ta.lds");
        let link_script = self.generate_new_link_script(ta_dev_kit_dir)?;
        if !std::fs::read(link_script_dest.as_path())
            .is_ok_and(|v| v.as_slice() == link_script.as_bytes())
        {
            std::fs::write(link_script_dest.as_path(), link_script.as_bytes())?;
        }

        Self::change_default_page_size();
        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rerun-if-changed={}", link_script_dest.display());
        println!("cargo:rustc-link-arg=-T{}", link_script_dest.display());
        Ok(())
    }

    // Correcting ELF segment alignment discrepancy between Rust and C, and in
    // the linker script provided by OP-TEE, the alignment are set to 0x1000.
    //
    // There is a mismatch in the ELF segment alignment between the Rust and
    // C builds.
    // The C-compiled binary correctly uses an alignment of 0x1000 (4KB),
    // which is required for OP-TEE compatibility. However, the
    // Rust-generated ELF defaults to 0x10000 (64KB).
    // To resolve this, we need to adjust the Rust linker parameters to
    // match the C alignment.
    //
    // example of C build elf header:
    //  Elf file type is DYN (Position-Independent Executable file)
    //  Entry point 0x2f4
    //  There are 4 program headers, starting at offset 64
    //
    //  Program Headers:
    //    Type           Offset             VirtAddr           PhysAddr
    //                   FileSiz            MemSiz              Flags  Align
    //    LOAD           0x0000000000001000 0x0000000000000000 0x0000000000000000
    //                   0x00000000000191b8 0x00000000000191b8  R E    0x1000
    //    LOAD           0x000000000001b000 0x000000000001a000 0x000000000001a000
    //                   0x00000000000006c4 0x000000000000bf80  RW     0x1000
    //    DYNAMIC        0x000000000001b000 0x000000000001a000 0x000000000001a000
    //                   0x0000000000000110 0x0000000000000110  RW     0x8
    //    GNU_STACK      0x0000000000000000 0x0000000000000000 0x0000000000000000
    //                   0x0000000000000000 0x0000000000000000  RW     0x10
    //
    // example of Rust build elf header:
    //  Elf file type is DYN (Position-Independent Executable file)
    //  Entry point 0x18d8
    //  There are 5 program headers, starting at offset 64
    //
    //  Program Headers:
    //    Type           Offset             VirtAddr           PhysAddr
    //                   FileSiz            MemSiz              Flags  Align
    //    LOAD           0x0000000000010000 0x0000000000000000 0x0000000000000000
    //                   0x000000000001c89c 0x0000000000028150  RWE    0x10000
    //    DYNAMIC        0x000000000002c000 0x000000000001c000 0x000000000001c000
    //                   0x0000000000000170 0x0000000000000170  RW     0x8
    //    NOTE           0x000000000002c4b0 0x000000000001c4b0 0x000000000001c4b0
    //                   0x0000000000000044 0x0000000000000044  R      0x4
    //    GNU_EH_FRAME   0x0000000000024b54 0x0000000000014b54 0x0000000000014b54
    //                   0x0000000000000e6c 0x0000000000000e6c  R      0x4
    //    GNU_STACK      0x0000000000000000 0x0000000000000000 0x0000000000000000
    //                   0x0000000000000000 0x0000000000000000  RW     0x10
    fn change_default_page_size() {
        println!("cargo:rustc-link-arg=-z");
        println!("cargo:rustc-link-arg=max-page-size=0x1000");
        println!("cargo:rustc-link-arg=-z");
        println!("cargo:rustc-link-arg=common-page-size=0x1000");
    }

    fn generate_new_link_script(&self, ta_dev_kit_dir: PathBuf) -> Result<String, Error> {
        let link_script_template_path = ta_dev_kit_dir.join("src/ta.ld.S");
        println!("cargo:rerun-if-changed={}", link_script_template_path.display());

        let link_script_output = {
            const ENV_CC: &str = "CC";
            println!("cargo:rerun-if-env-changed={}", ENV_CC);

            let cc_cmd = env::var(ENV_CC).unwrap_or("cc".to_string());
            let mut tmp = Command::new(cc_cmd);
            tmp.args([
                "-E",
                "-P",
                "-x",
                "c",
                link_script_template_path.to_str().expect("infallible"),
            ]);
            const ENV_TARGET_ARCH: &str = "CARGO_CFG_TARGET_ARCH";
            println!("cargo:rerun-if-env-changed={}", ENV_TARGET_ARCH);
            match env::var(ENV_TARGET_ARCH)?.as_str() {
                "riscv32" => {
                    tmp.arg("-DRV32=1");
                }
                "riscv64" => {
                    tmp.arg("-DRV64=1");
                }
                "arm" => {
                    tmp.arg("-DARM32=1");
                }
                "aarch64" => {
                    tmp.arg("-DARM64=1");
                }
                _ => {}
            };
            if let Some(ftrace_buf_size) = self.ftrace_buf_size {
                tmp.arg(format!("-DCFG_FTRACE_BUF_SIZE={}", ftrace_buf_size));
            }
            tmp
        }
        .output()?
        .stdout;
        let link_script_text = String::from_utf8(link_script_output)?;
        Ok(link_script_text)
    }

    fn auto_detect_linker_type() -> LinkerType {
        const ENV_RUSTC_LINKER: &str = "RUSTC_LINKER";
        println!("cargo:rerun-if-env-changed={}", ENV_RUSTC_LINKER);
        match env::var(ENV_RUSTC_LINKER) {
            Ok(ref linker_name)
                if linker_name.ends_with("ld") || linker_name.ends_with("ld.bfd") =>
            {
                LinkerType::Ld
            }
            _ => LinkerType::Cc,
        }
    }
}
