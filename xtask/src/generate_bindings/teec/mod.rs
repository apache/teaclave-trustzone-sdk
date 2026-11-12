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

//! `generate-bindings teec`: produce `crates/optee-teec-sys/src/generated.rs`
//! from the OP-TEE client headers.

mod generate;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use super::{LICENSE_HEADER, TARGET};
use generate::generate;

/// Note placed after the license header.
const GENERATED_HEADER: &str = include_str!("generated_header.in");

/// Committed bindings, relative to the repository root.
const OUTPUT: &str = "crates/optee-teec-sys/src/generated.rs";

#[derive(clap::Args)]
pub(super) struct TeecArgs {
    /// OP-TEE client export directory (the one containing `usr/include/`).
    #[arg(long, env = "OPTEE_CLIENT_EXPORT")]
    optee_client_dir: Option<PathBuf>,
}

pub(super) fn run(args: TeecArgs) -> Result<(), Box<dyn Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask has no parent directory")?;

    let optee_client_dir = args
        .optee_client_dir
        .ok_or("OPTEE_CLIENT_EXPORT is not set; pass --optee-client-dir")?;
    let include_dir = optee_client_dir.join("usr/include");
    if !include_dir.is_dir() {
        return Err(format!(
            "{} is not a directory; check OPTEE_CLIENT_EXPORT",
            include_dir.display()
        )
        .into());
    }

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
