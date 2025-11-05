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

use anyhow::{bail, Result};
use clap::ValueEnum;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use toml::Value;

/// Target architecture for building
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Arch {
    /// ARM 64-bit architecture
    Aarch64,
    /// ARM 32-bit architecture
    Arm,
}

impl std::str::FromStr for Arch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aarch64" | "arm64" => Ok(Arch::Aarch64),
            "arm" | "arm32" => Ok(Arch::Arm),
            _ => Err(format!("Invalid architecture: {}", s)),
        }
    }
}

/// Helper function to print command output and return error
pub fn print_output_and_bail(cmd_name: &str, output: &Output) -> Result<()> {
    eprintln!(
        "{} stdout: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "{} stderr: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stderr)
    );
    bail!(
        "{} failed with exit code: {:?}",
        cmd_name,
        output.status.code()
    )
}

/// Helper function to derive target and cross-compile prefix from arch
pub fn get_target_and_cross_compile(arch: Arch) -> (String, String) {
    match arch {
        Arch::Arm => (
            "arm-unknown-linux-gnueabihf".to_string(),
            "arm-linux-gnueabihf-".to_string(),
        ),
        Arch::Aarch64 => (
            "aarch64-unknown-linux-gnu".to_string(),
            "aarch64-linux-gnu-".to_string(),
        ),
    }
}

/// RAII guard to ensure we return to the original directory
pub struct ChangeDirectoryGuard {
    original: PathBuf,
}

impl ChangeDirectoryGuard {
    pub fn new(new_dir: &PathBuf) -> Result<Self> {
        let original = env::current_dir()?;
        env::set_current_dir(new_dir)?;
        Ok(Self { original })
    }
}

impl Drop for ChangeDirectoryGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

/// Print cargo command for debugging
pub fn print_cargo_command(cmd: &Command, description: &str) {
    println!("{}...", description);

    // Extract program and args
    let program = cmd.get_program();
    let args: Vec<_> = cmd.get_args().collect();

    // Extract all environment variables
    let envs: Vec<String> = cmd
        .get_envs()
        .filter_map(|(k, v)| match (k.to_str(), v.and_then(|v| v.to_str())) {
            (Some(key), Some(value)) => Some(format!("{}={}", key, value)),
            _ => None,
        })
        .collect();

    // Print environment variables
    if !envs.is_empty() {
        println!("  Environment: {}", envs.join(" "));
    }

    // Print command
    println!(
        "  Command: {} {}",
        program.to_string_lossy(),
        args.into_iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
}

/// Find the target directory using Cargo's workspace discovery strategy
/// Start from current directory and walk up looking for workspace root
pub fn find_target_directory() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;

    loop {
        // Check if current directory has a Cargo.toml that declares a workspace
        let cargo_toml_path = current_dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;
            if let Ok(cargo_toml) = toml::from_str::<Value>(&cargo_toml_content) {
                // If this Cargo.toml has a [workspace] section, this is the workspace root
                if cargo_toml.get("workspace").is_some() {
                    return Ok(current_dir.join("target"));
                }
            }
        }

        // Move to parent directory
        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            // Reached filesystem root, no workspace found
            // Use target directory in the original crate directory
            return Ok(env::current_dir()?.join("target"));
        }
    }
}

/// Read UUID from a file (e.g., uuid.txt)
pub fn read_uuid_from_file(uuid_path: &std::path::Path) -> Result<String> {
    if !uuid_path.exists() {
        bail!("UUID file not found: {}", uuid_path.display());
    }

    let uuid_content = fs::read_to_string(uuid_path)?;
    let uuid = uuid_content.trim().to_string();

    if uuid.is_empty() {
        bail!("UUID file is empty: {}", uuid_path.display());
    }

    Ok(uuid)
}
