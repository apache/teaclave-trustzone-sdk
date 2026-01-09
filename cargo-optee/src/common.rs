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
use serde_json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use toml::Value;

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

/// Build mode for OP-TEE components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Client Application (CA) - runs in Normal World Linux environment
    /// Can use standard library, uses standard Linux targets
    Ca,
    /// Trusted Application with std support
    /// Uses OP-TEE custom targets (e.g., aarch64-unknown-optee)
    TaStd,
    /// Trusted Application without std support
    /// Uses standard Linux targets but runs in TEE environment
    TaNoStd,
}

/// Target configurations for different architectures and build modes
/// Format: (Architecture, BuildMode, target, cross_compile_prefix)
const TARGET_CONFIGS: [(Arch, BuildMode, &str, &str); 6] = [
    // ARM 32-bit configurations
    (
        Arch::Arm,
        BuildMode::Ca,
        "arm-unknown-linux-gnueabihf",
        "arm-linux-gnueabihf-",
    ),
    (
        Arch::Arm,
        BuildMode::TaNoStd,
        "arm-unknown-linux-gnueabihf",
        "arm-linux-gnueabihf-",
    ),
    (
        Arch::Arm,
        BuildMode::TaStd,
        "arm-unknown-optee",
        "arm-linux-gnueabihf-",
    ),
    // AArch64 configurations
    (
        Arch::Aarch64,
        BuildMode::Ca,
        "aarch64-unknown-linux-gnu",
        "aarch64-linux-gnu-",
    ),
    (
        Arch::Aarch64,
        BuildMode::TaNoStd,
        "aarch64-unknown-linux-gnu",
        "aarch64-linux-gnu-",
    ),
    (
        Arch::Aarch64,
        BuildMode::TaStd,
        "aarch64-unknown-optee",
        "aarch64-linux-gnu-",
    ),
];

/// Unified function to derive target and cross-compile prefix from architecture and build mode
pub fn get_target_and_cross_compile(arch: Arch, mode: BuildMode) -> Result<(String, String)> {
    for &(config_arch, config_mode, target, cross_compile_prefix) in &TARGET_CONFIGS {
        if config_arch == arch && config_mode == mode {
            return Ok((target.to_string(), cross_compile_prefix.to_string()));
        }
    }

    bail!(
        "No target configuration found for arch: {:?}, mode: {:?}",
        arch,
        mode
    )
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

/// Get the target directory using cargo metadata
pub fn get_target_directory_from_metadata() -> Result<PathBuf> {
    // We're already in the project directory, so no need for --manifest-path
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .output()?;

    if !output.status.success() {
        bail!("Failed to get cargo metadata");
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let target_directory = metadata
        .get("target_directory")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Could not get target directory from cargo metadata"))?;

    Ok(PathBuf::from(target_directory))
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

/// Join path segments and check if the resulting path exists
pub fn join_and_check<P: AsRef<std::path::Path>>(
    base: &std::path::Path,
    segments: &[P],
    error_context: &str,
) -> Result<PathBuf> {
    let mut path = base.to_path_buf();
    for segment in segments {
        path = path.join(segment);
    }

    if !path.exists() {
        bail!("{} does not exist: {:?}", error_context, path);
    }

    Ok(path)
}

/// Join path segments with a formatted final segment and check if the resulting path exists
pub fn join_format_and_check<P: AsRef<std::path::Path>>(
    base: &std::path::Path,
    segments: &[P],
    formatted_segment: &str,
    error_context: &str,
) -> Result<PathBuf> {
    let mut path = base.to_path_buf();
    for segment in segments {
        path = path.join(segment);
    }

    let final_path = path.join(formatted_segment);

    if !final_path.exists() {
        bail!("{} does not exist: {:?}", error_context, final_path);
    }

    Ok(final_path)
}

/// Clean build artifacts for any OP-TEE component (TA, CA, Plugin)
pub fn clean_project(project_path: &std::path::Path) -> Result<()> {
    println!("Cleaning build artifacts in: {:?}", project_path);

    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        print_output_and_bail("cargo clean", &output)?;
    }

    // Also clean the intermediate cargo-optee directory if it exists
    let intermediate_dir = project_path.join("target").join("cargo-optee");
    if intermediate_dir.exists() {
        fs::remove_dir_all(&intermediate_dir)?;
        println!("Removed intermediate directory: {:?}", intermediate_dir);
    }

    println!("Build artifacts cleaned successfully");
    Ok(())
}

/// Get the package name from Cargo.toml in the current directory
pub fn get_package_name() -> Result<String> {
    // We assume we're already in the project directory (via ChangeDirectoryGuard)
    let manifest_path = PathBuf::from("Cargo.toml");
    if !manifest_path.exists() {
        bail!("Cargo.toml not found in current directory");
    }

    let cargo_toml_content = fs::read_to_string(&manifest_path)?;
    let cargo_toml: Value = toml::from_str(&cargo_toml_content)?;

    let package_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))?;

    Ok(package_name.to_string())
}
