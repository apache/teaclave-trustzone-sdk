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

use crate::common;
use anyhow::{bail, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

use crate::common::{
    get_package_name, get_target_and_cross_compile, get_target_directory_from_metadata,
    print_cargo_command, print_output_and_bail, read_uuid_from_file, BuildMode,
    ChangeDirectoryGuard,
};
use crate::config::TaBuildConfig;

// Embed the target JSON files at compile time
const AARCH64_TARGET_JSON: &str = include_str!("../aarch64-unknown-optee.json");
const ARM_TARGET_JSON: &str = include_str!("../arm-unknown-optee.json");

// Main function to build the TA, optionally installing to a target directory
pub fn build_ta(config: TaBuildConfig, install_dir: Option<&Path>) -> Result<()> {
    // Verify we're in a valid Rust project directory
    let manifest_path = config.path.join("Cargo.toml");
    if !manifest_path.exists() {
        bail!(
            "No Cargo.toml found in TA project directory: {:?}\n\
            Please run cargo-optee from a TA project directory or specify --manifest-path",
            config.path
        );
    }

    // Change to the TA directory (RAII guard ensures we return to original directory)
    let _guard = ChangeDirectoryGuard::new(&config.path)?;

    // Check if required cross-compile toolchain is available
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (_, cross_compile_prefix) = get_target_and_cross_compile(config.arch, build_mode)?;
    check_toolchain_exists(&cross_compile_prefix)?;

    // Get the absolute path for better clarity
    let absolute_path = std::fs::canonicalize(&config.path).unwrap_or_else(|_| config.path.clone());
    println!("Building TA in directory: {}", absolute_path.display());

    // Step 1: Run clippy for code quality checks
    run_clippy(&config)?;

    // Step 2: Build the TA
    build_binary(&config)?;

    // Step 3: Strip the binary
    let (stripped_path, target_dir) = strip_binary(&config)?;

    // Step 4: Sign the TA
    sign_ta(&config, &stripped_path, &target_dir)?;

    // Step 5: Install if requested
    if let Some(install_dir) = install_dir {
        // Check if install directory exists
        if !install_dir.exists() {
            bail!("Install directory does not exist: {:?}", install_dir);
        }

        let uuid_path = config
            .uuid_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UUID path is required but not configured"))?;
        let uuid = read_uuid_from_file(uuid_path)?;
        let ta_file = common::join_format_and_check::<&str>(
            &target_dir,
            &[],
            &format!("{}.ta", uuid),
            "Signed TA file",
        )?;

        let dest_path = install_dir.join(format!("{}.ta", uuid));
        fs::copy(&ta_file, &dest_path)?;

        println!(
            "TA installed to: {:?}",
            dest_path.canonicalize().unwrap_or(dest_path)
        );
    }

    println!("TA build successfully!");

    Ok(())
}

fn run_clippy(config: &TaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    // Run cargo fmt (we're already in the project directory via ChangeDirectoryGuard)
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
    // Determine target and cross-compile based on arch and std mode
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, build_mode)?;

    // Setup build command with common environment (we're already in the project directory)
    let (mut build_cmd, _temp_dir) = setup_build_command(config, "build")?;

    if !config.debug {
        build_cmd.arg("--release");
    }

    // Configure linker
    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);

    // Print the full cargo build command for debugging
    print_cargo_command(&build_cmd, "Building TA binary");

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        print_output_and_bail("build", &build_output)?;
    }

    Ok(())
}

fn strip_binary(config: &TaBuildConfig) -> Result<(PathBuf, PathBuf)> {
    println!("Stripping binary...");

    // Determine target based on arch and std mode
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, build_mode)?;

    let profile = if config.debug { "debug" } else { "release" };

    // Use cargo metadata to get the target directory (supports workspace and CARGO_TARGET_DIR)
    let target_directory = get_target_directory_from_metadata()?;
    let profile_dir = target_directory.join(target).join(profile);

    // Get the actual package name from Cargo.toml (we're already in the project directory)
    let package_name = get_package_name()?;

    let binary_path = common::join_and_check(&profile_dir, &[&package_name], "Binary")?;

    let stripped_path = profile_dir.join(format!("stripped_{}", package_name));

    let objcopy = format!("{}objcopy", cross_compile);

    let strip_output = Command::new(&objcopy)
        .arg("--strip-unneeded")
        .arg(&binary_path)
        .arg(&stripped_path)
        .output()?;

    if !strip_output.status.success() {
        print_output_and_bail(&objcopy, &strip_output)?;
    }

    Ok((stripped_path, profile_dir))
}

fn sign_ta(config: &TaBuildConfig, stripped_path: &Path, target_dir: &Path) -> Result<()> {
    println!("Signing TA with signing key {:?}...", config.signing_key);

    // Read UUID from specified file
    let uuid_path = config
        .uuid_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("UUID path is required but not configured"))?;
    let uuid = read_uuid_from_file(uuid_path)?;

    // Validate signing key exists
    if !config.signing_key.exists() {
        bail!("Signing key not found at {:?}", config.signing_key);
    }

    // Sign script path
    let sign_script = common::join_and_check(
        &config.ta_dev_kit_dir,
        &["scripts", "sign_encrypt.py"],
        "Sign script",
    )?;

    // Output path - use the actual target_dir
    let output_path = target_dir.join(format!("{}.ta", uuid));

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
    let absolute_output_path = output_path.canonicalize().unwrap_or(output_path);
    println!("TA signed and saved to: {:?}", absolute_output_path);

    Ok(())
}

/// Check if the required cross-compile toolchain is available
fn check_toolchain_exists(cross_compile_prefix: &str) -> Result<()> {
    let gcc_command = format!("{}gcc", cross_compile_prefix);
    let objcopy_command = format!("{}objcopy", cross_compile_prefix);

    // Check if gcc exists
    let gcc_check = Command::new("which").arg(&gcc_command).output();

    // Check if objcopy exists
    let objcopy_check = Command::new("which").arg(&objcopy_command).output();

    let gcc_exists = gcc_check.map_or(false, |output| output.status.success());
    let objcopy_exists = objcopy_check.map_or(false, |output| output.status.success());

    if !gcc_exists || !objcopy_exists {
        let missing_tools: Vec<&str> = [
            if !gcc_exists {
                Some(gcc_command.as_str())
            } else {
                None
            },
            if !objcopy_exists {
                Some(objcopy_command.as_str())
            } else {
                None
            },
        ]
        .iter()
        .filter_map(|&x| x)
        .collect();

        eprintln!("Error: Required cross-compile toolchain not found!");
        eprintln!("Missing tools: {}", missing_tools.join(", "));
        eprintln!();
        eprintln!("Please install the required toolchain:");
        eprintln!();
        eprintln!("# For aarch64 host (ARM64 machine):");
        eprintln!("apt update && apt -y install gcc gcc-arm-linux-gnueabihf");
        eprintln!();
        eprintln!("# For x86_64 host (Intel/AMD machine):");
        eprintln!("apt update && apt -y install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf");
        eprintln!();
        eprintln!("Or manually install the cross-compilation tools for your target architecture.");

        bail!("Cross-compile toolchain not available");
    }

    Ok(())
}

// Helper function to setup base command with common environment variables
fn setup_build_command(
    config: &TaBuildConfig,
    command: &str,
) -> Result<(Command, Option<TempDir>)> {
    // Determine target and cross-compile based on arch and std mode
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (target, _cross_compile) = get_target_and_cross_compile(config.arch, build_mode)?;

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

    // Add --no-default-features if specified
    if config.no_default_features {
        cmd.arg("--no-default-features");
    }

    // Build features list
    let mut features = Vec::new();
    if config.std {
        features.push("std".to_string());
    }
    if let Some(ref custom_features) = config.features {
        // Split custom features by comma and add them
        for feature in custom_features.split(',') {
            let feature = feature.trim();
            if !feature.is_empty() {
                features.push(feature.to_string());
            }
        }
    }

    // Add features if any are specified
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    // Add no-std specific flags to avoid the linking error of _Unwind_Resume
    if !config.std {
        cmd.arg("-Z").arg("build-std=core,alloc");
        cmd.arg("-Z")
            .arg("build-std-features=panic_immediate_abort");
    }

    // Set RUSTFLAGS - preserve existing ones and add panic=abort
    let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.is_empty() {
        rustflags.push(' ');
    }
    rustflags.push_str("-C panic=abort");
    cmd.env("RUSTFLAGS", &rustflags);

    // Apply custom environment variables
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    // Set TA_DEV_KIT_DIR environment variable (use absolute path)
    let absolute_ta_dev_kit_dir = config
        .ta_dev_kit_dir
        .canonicalize()
        .unwrap_or_else(|_| config.ta_dev_kit_dir.clone());
    cmd.env("TA_DEV_KIT_DIR", &absolute_ta_dev_kit_dir);

    // Set RUST_TARGET_PATH for custom targets when using std
    if let Some(ref temp_dir_ref) = temp_dir {
        cmd.env("RUST_TARGET_PATH", temp_dir_ref.path());
    }

    Ok((cmd, temp_dir))
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
