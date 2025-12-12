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
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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
}

impl Linker {
    /// Construct a Linker by manually specific the type of linker, you may use
    /// `auto`, it would detect current linker automatically.
    pub fn new(linker_type: LinkerType) -> Self {
        Self { linker_type }
    }
    /// Construct a Linker by auto detect the type of linker, try `new` function
    ///  if our detection mismatch.
    pub fn auto() -> Self {
        Self {
            linker_type: Self::auto_detect_linker_type(),
        }
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
        write!(
            dyn_list,
            "{{ __elf_phdr_info; trace_ext_prefix; trace_level; ta_head; }};\n"
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
        let mut aarch64_flag = true;
        match env::var(ENV_TARGET) {
            Ok(ref v) if v == "arm-unknown-linux-gnueabihf" || v == "arm-unknown-optee" => {
                match self.linker_type {
                    LinkerType::Cc => println!("cargo:rustc-link-arg=-Wl,--no-warn-mismatch"),
                    LinkerType::Ld => println!("cargo:rustc-link-arg=--no-warn-mismatch"),
                };
                aarch64_flag = false;
            }
            _ => {}
        };

        let f = BufReader::new(File::open(ta_dev_kit_dir.join("src/ta.ld.S"))?);
        let ta_lds_file_path = out_dir.join("ta.lds");
        let mut ta_lds = File::create(ta_lds_file_path.clone())?;
        for line in f.lines() {
            let l = line?;

            if aarch64_flag {
                if l.starts_with('#')
                    || l == "OUTPUT_FORMAT(\"elf32-littlearm\")"
                    || l == "OUTPUT_ARCH(arm)"
                {
                    continue;
                }
            } else {
                if l.starts_with('#')
                    || l == "OUTPUT_FORMAT(\"elf64-littleaarch64\")"
                    || l == "OUTPUT_ARCH(aarch64)"
                {
                    continue;
                }
            }

            if l == "\t. = ALIGN(4096);" {
                write!(ta_lds, "\t. = ALIGN(65536);\n")?;
            } else {
                write!(ta_lds, "{}\n", l)?;
            }
        }

        println!("cargo:rustc-link-search={}", out_dir.display());
        println!("cargo:rerun-if-changed={}", ta_lds_file_path.display());
        println!("cargo:rustc-link-arg=-T{}", ta_lds_file_path.display());
        Ok(())
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
