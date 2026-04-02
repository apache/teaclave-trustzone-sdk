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

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::Error;
use crate::HeaderFileGenerator;
use crate::RustEdition;
use crate::TaConfig;
use crate::{Linker, LinkerType};

const DEFAULT_HEADER_FILE_NAME: &str = "user_ta_header.rs";

/// The Builder of TA, use it to handle file generation and linking stuff
///
/// Usage:
///
/// ```no_run
/// use optee_utee_build::{TaConfig, Builder, RustEdition};
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default(UUID, "0.1.0", "example")?;
/// Builder::new(RustEdition::Before2024, ta_config).build()?;
/// # Ok(())
/// # }
/// ```
///
/// Or you can just use the build method, it's simpler
/// ```no_run
/// use optee_utee_build::{TaConfig, RustEdition};
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default(UUID, "0.1.0", "example")?;
/// optee_utee_build::build(RustEdition::Before2024, ta_config)?;
/// # Ok(())
/// # }
/// ```
///
/// There are some difference when cargo use different linkers, we will try
/// to detect the linker automatically, you can set it manually if you met
/// some problems with it.
/// ```no_run
/// use optee_utee_build::{TaConfig, Builder, RustEdition, LinkerType};
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default(UUID, "0.1.0", "example")?;
/// Builder::new(RustEdition::Before2024, ta_config).linker_type(LinkerType::Ld).build()?;
/// # Ok(())
/// # }
/// ```
pub struct Builder {
    out_dir: Option<PathBuf>,
    edition: RustEdition,
    header_file_name: Option<String>,
    ta_config: TaConfig,
    linker_type: Option<LinkerType>,
}

impl Builder {
    pub fn new(edition: RustEdition, ta_config: TaConfig) -> Self {
        Self {
            out_dir: Option::None,
            header_file_name: Option::None,
            linker_type: Option::None,
            edition,
            ta_config,
        }
    }
    pub fn out_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.out_dir = Option::Some(path.into());
        self
    }
    pub fn header_file_name<S: Into<String>>(mut self, file_name: S) -> Self {
        self.header_file_name = Option::Some(file_name.into());
        self
    }
    pub fn linker_type(mut self, linker_type: LinkerType) -> Self {
        self.linker_type = Option::Some(linker_type);
        self
    }
    pub fn build(self) -> Result<(), Error> {
        let out_dir = match self.out_dir.clone() {
            Some(v) => v,
            None => PathBuf::from(std::env::var("OUT_DIR")?),
        };
        self.write_header_file(out_dir.clone())?;
        self.link(out_dir)?;
        Ok(())
    }
}

impl Builder {
    fn write_header_file(&self, out: PathBuf) -> Result<(), Error> {
        let out_header_file_name = out.join(match self.header_file_name.as_ref() {
            Some(v) => v.as_str(),
            None => DEFAULT_HEADER_FILE_NAME,
        });
        let mut buffer = File::create(out_header_file_name.clone())?;
        let header_codes =
            HeaderFileGenerator::new(self.edition.clone()).generate(&self.ta_config)?;
        buffer.write_all(header_codes.as_bytes())?;
        Ok(())
    }

    fn link(&self, out_dir: PathBuf) -> Result<(), Error> {
        let linker = match self.linker_type.as_ref() {
            Option::Some(v) => Linker::new(v.clone()),
            Option::None => Linker::auto(),
        };
        linker.link_all(out_dir)
    }
}
