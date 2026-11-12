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

//! `generate-bindings utee`: produce `crates/optee-utee-sys/src/generated.rs`
//! from the OP-TEE TA dev kit headers.

mod generate;
mod verify;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use super::{LICENSE_HEADER, TARGET};
use generate::generate;
use verify::check_optee_version;

/// Note placed after the license header.
const GENERATED_HEADER: &str = include_str!("generated_header.in");

/// Committed bindings, relative to the repository root.
const OUTPUT: &str = "crates/optee-utee-sys/src/generated.rs";

#[derive(clap::Args)]
pub(super) struct UteeArgs {
    /// OP-TEE TA dev kit directory (the one containing `include/`).
    #[arg(long, env = "TA_DEV_KIT_DIR")]
    ta_dev_kit_dir: Option<PathBuf>,
}

pub(super) fn run(args: UteeArgs) -> Result<(), Box<dyn Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask has no parent directory")?;

    let version = fs::read_to_string(repo_root.join("optee-version.txt"))?;
    let version = version.trim();
    if version.is_empty() {
        return Err("optee-version.txt is empty".into());
    }

    let ta_dev_kit_dir = args
        .ta_dev_kit_dir
        .ok_or("TA_DEV_KIT_DIR is not set; pass --ta-dev-kit-dir")?;
    let include_dir = ta_dev_kit_dir.join("include");
    if !include_dir.is_dir() {
        return Err(format!(
            "{} is not a directory; check TA_DEV_KIT_DIR",
            include_dir.display()
        )
        .into());
    }

    check_optee_version(version, &ta_dev_kit_dir)?;

    let bindings = generate(&include_dir, TARGET)?;

    let mut output = String::new();
    output.push_str(LICENSE_HEADER);
    output.push('\n');
    output.push_str(GENERATED_HEADER);
    output.push('\n');
    output.push_str(&bindings);

    let output_path = repo_root.join(OUTPUT);

    fs::write(&output_path, output)?;
    println!("wrote {}", output_path.display());

    Ok(())
}
