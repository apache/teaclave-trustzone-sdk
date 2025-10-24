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
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

use crate::common::{print_output_and_bail, Arch, ChangeDirectoryGuard};

// Embed the target JSON files at compile time
const AARCH64_TARGET_JSON: &str = include_str!("../aarch64-unknown-optee.json");
const ARM_TARGET_JSON: &str = include_str!("../arm-unknown-optee.json");

pub struct TaBuildConfig {
    pub arch: Arch,              // Architecture
    pub std: bool,               // Enable std feature
    pub ta_dev_kit_dir: PathBuf, // Path to TA dev kit
    pub signing_key: PathBuf,    // Path to signing key
    pub uuid_path: PathBuf,      // Path to uuid.txt
    pub debug: bool,             // Debug mode (default false = release)
    pub path: PathBuf,           // Path to TA directory
}

// Helper function to derive target and cross-compile from arch and std
fn get_target_and_cross_compile(arch: Arch, std: bool) -> (String, String) {
    match arch {
        Arch::Arm => {
            if std {
                (
                    "arm-unknown-optee".to_string(),
                    "arm-linux-gnueabihf-".to_string(),
                )
            } else {
                (
                    "arm-unknown-linux-gnueabihf".to_string(),
                    "arm-linux-gnueabihf-".to_string(),
                )
            }
        }
        Arch::Aarch64 => {
            if std {
                (
                    "aarch64-unknown-optee".to_string(),
                    "aarch64-linux-gnu-".to_string(),
                )
            } else {
                (
                    "aarch64-unknown-linux-gnu".to_string(),
                    "aarch64-linux-gnu-".to_string(),
                )
            }
        }
    }
}

// Helper function to setup custom target JSONs for std builds
// Returns TempDir to keep it alive during the build
fn setup_custom_targets() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Write the embedded target JSON files
    let aarch64_path = temp_dir.path().join("aarch64-unknown-optee.json");
    let arm_path = temp_dir.path().join("arm-unknown-optee.json");

    fs::write(aarch64_path, AARCH64_TARGET_JSON)?;
    fs::write(arm_path, ARM_TARGET_JSON)?;

    Ok(temp_dir)
}

// Helper function to setup base command with common environment variables
fn setup_build_command(
    config: &TaBuildConfig,
    command: &str,
) -> Result<(Command, Option<TempDir>)> {
    // Determine target and cross-compile based on arch
    let (target, _cross_compile) = get_target_and_cross_compile(config.arch, config.std);

    // Determine builder (cargo or xargo)
    let builder = if config.std { "xargo" } else { "cargo" };

    // Setup custom targets if using std - keep TempDir alive
    let temp_dir = if config.std {
        Some(setup_custom_targets()?)
    } else {
        None
    };

    let mut cmd = Command::new(builder);
    cmd.arg(command);
    cmd.arg("--target").arg(&target);

    // Add std feature if enabled
    if config.std {
        cmd.arg("--features").arg("std");
    }

    // Set RUSTFLAGS - preserve existing ones and add panic=abort
    let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.is_empty() {
        rustflags.push(' ');
    }
    rustflags.push_str("-C panic=abort");
    cmd.env("RUSTFLAGS", &rustflags);

    // Set TA_DEV_KIT_DIR environment variable
    cmd.env("TA_DEV_KIT_DIR", &config.ta_dev_kit_dir);

    // Set RUST_TARGET_PATH for custom targets when using std
    if let Some(ref temp_dir_ref) = temp_dir {
        cmd.env("RUST_TARGET_PATH", temp_dir_ref.path());
    }

    Ok((cmd, temp_dir))
}

// Main function to build the TA
pub fn build_ta(config: TaBuildConfig) -> Result<()> {
    // Change to the TA directory
    let _guard = ChangeDirectoryGuard::new(&config.path)?;

    println!("Building TA in directory: {:?}", config.path);

    // Step 1: Run clippy for code quality checks
    run_clippy(&config)?;

    // Step 2: Build the TA
    build_binary(&config)?;

    // Step 3: Strip the binary
    let stripped_path = strip_binary(&config)?;

    // Step 4: Sign the TA
    sign_ta(&config, &stripped_path)?;

    println!("TA build completed successfully!");

    Ok(())
}

fn run_clippy(config: &TaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    // Run cargo fmt
    let fmt_output = Command::new("cargo").arg("fmt").output()?;

    if !fmt_output.status.success() {
        print_output_and_bail("cargo fmt", &fmt_output)?;
    }

    // Setup clippy command with common environment
    let (mut clippy_cmd, _temp_dir) = setup_build_command(config, "clippy")?;

    clippy_cmd.arg("--");
    clippy_cmd.arg("-D").arg("warnings");
    clippy_cmd.arg("-D").arg("clippy::unwrap_used");
    clippy_cmd.arg("-D").arg("clippy::expect_used");
    clippy_cmd.arg("-D").arg("clippy::panic");

    let clippy_output = clippy_cmd.output()?;

    if !clippy_output.status.success() {
        print_output_and_bail("clippy", &clippy_output)?;
    }

    Ok(())
}

fn build_binary(config: &TaBuildConfig) -> Result<()> {
    println!("Building TA binary...");

    // Determine target and cross-compile based on arch
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, config.std);

    // Setup build command with common environment
    let (mut build_cmd, _temp_dir) = setup_build_command(config, "build")?;

    if !config.debug {
        build_cmd.arg("--release");
    }

    // Configure linker
    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        print_output_and_bail("build", &build_output)?;
    }

    Ok(())
}

fn strip_binary(config: &TaBuildConfig) -> Result<PathBuf> {
    println!("Stripping binary...");

    // Determine target based on arch
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, config.std);

    let profile = if config.debug { "debug" } else { "release" };
    let target_dir = PathBuf::from("target").join(target).join(profile);

    let binary_path = target_dir.join("ta");
    let stripped_path = target_dir.join("stripped_ta");

    if !binary_path.exists() {
        bail!("Binary not found at {:?}", binary_path);
    }

    let objcopy = format!("{}objcopy", cross_compile);

    let strip_output = Command::new(&objcopy)
        .arg("--strip-unneeded")
        .arg(&binary_path)
        .arg(&stripped_path)
        .output()?;

    if !strip_output.status.success() {
        print_output_and_bail(&objcopy, &strip_output)?;
    }

    Ok(stripped_path)
}

fn sign_ta(config: &TaBuildConfig, stripped_path: &Path) -> Result<()> {
    println!("Signing TA...");

    // Read UUID from the specified path
    let uuid = fs::read_to_string(&config.uuid_path)?.trim().to_string();

    // Validate signing key exists
    if !config.signing_key.exists() {
        bail!("Signing key not found at {:?}", config.signing_key);
    }

    // Sign script path
    let sign_script = config
        .ta_dev_kit_dir
        .join("scripts")
        .join("sign_encrypt.py");
    if !sign_script.exists() {
        bail!("Sign script not found at {:?}", sign_script);
    }

    // Determine target based on arch
    let (target, _) = get_target_and_cross_compile(config.arch, config.std);

    // Output path
    let profile = if config.debug { "debug" } else { "release" };
    let output_path = PathBuf::from("target")
        .join(target)
        .join(profile)
        .join(format!("{}.ta", uuid));

    let sign_output = Command::new("python3")
        .arg(&sign_script)
        .arg("--uuid")
        .arg(&uuid)
        .arg("--key")
        .arg(&config.signing_key)
        .arg("--in")
        .arg(stripped_path)
        .arg("--out")
        .arg(&output_path)
        .output()?;

    if !sign_output.status.success() {
        print_output_and_bail("sign_encrypt.py", &sign_output)?;
    }

    println!("SIGN => {}", uuid);
    println!("TA signed and saved to: {:?}", output_path);

    Ok(())
}
