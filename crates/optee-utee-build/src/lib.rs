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

mod builder;
mod code_generator;
mod error;
mod linker;
mod ta_config;

pub use builder::*;
pub use code_generator::*;
pub use error::Error;
pub use linker::*;
pub use ta_config::*;
pub use uuid::Uuid;

/// a build method, use it for TA compilation
/// Usage:
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
/// You may check the Builder struct if you need some customizations.
pub fn build(edition: RustEdition, config: TaConfig) -> Result<(), Error> {
    Builder::new(edition, config).build()
}
