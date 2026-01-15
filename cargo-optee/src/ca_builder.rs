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
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common;
use crate::common::{
    get_package_name, get_target_and_cross_compile, get_target_directory_from_metadata,
    print_cargo_command, print_output_and_bail, read_uuid_from_file, BuildMode,
    ChangeDirectoryGuard,
};
use crate::config::CaBuildConfig;

// Main function to build the CA, optionally installing to a target directory
pub fn build_ca(config: CaBuildConfig, install_dir: Option<&Path>) -> Result<()> {
    // Change to the CA directory
    let _guard = ChangeDirectoryGuard::new(&config.path)?;

    let component_type = if config.plugin { "Plugin" } else { "CA" };
    // Get the absolute path for better clarity
    let absolute_path = std::fs::canonicalize(&config.path).unwrap_or_else(|_| config.path.clone());
    println!(
        "Building {} in directory: {}",
        component_type,
        absolute_path.display()
    );

    // Step 1: Run clippy for code quality checks
    run_clippy(&config)?;

    // Step 2: Build the CA
    build_binary(&config)?;

    // Step 3: Post-build processing (strip for binaries, copy for plugins)
    let final_binary = post_build(&config)?;

    // Print the final binary path with descriptive prompt
    let absolute_final_binary = final_binary
        .canonicalize()
        .unwrap_or_else(|_| final_binary.clone());
    if config.plugin {
        println!("Plugin copied to: {}", absolute_final_binary.display());
    } else {
        println!(
            "CA binary stripped and saved to: {}",
            absolute_final_binary.display()
        );
    }

    // Step 4: Install if requested
    if let Some(install_dir) = install_dir {
        use std::fs;

        // Check if install directory exists
        if !install_dir.exists() {
            bail!("Install directory does not exist: {:?}", install_dir);
        }

        // Get package name from the final binary path
        let package_name = final_binary
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("Could not get binary name"))?;

        // Copy binary to install directory
        let dest_path = install_dir.join(package_name);
        fs::copy(&final_binary, &dest_path)?;

        println!(
            "{} installed to: {:?}",
            component_type,
            dest_path.canonicalize().unwrap_or(dest_path)
        );
    }

    println!("{} build successfully!", component_type);

    Ok(())
}

fn run_clippy(config: &CaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    // Run cargo fmt
    let fmt_output = Command::new("cargo").arg("fmt").output()?;

    if !fmt_output.status.success() {
        print_output_and_bail("cargo fmt", &fmt_output)?;
    }

    // Determine target based on arch (CA runs in Normal World Linux)
    let (target, _cross_compile) = get_target_and_cross_compile(config.arch, BuildMode::Ca)?;

    let mut clippy_cmd = Command::new("cargo");
    clippy_cmd.arg("clippy");
    clippy_cmd.arg("--target").arg(&target);

    // Set OPTEE_CLIENT_EXPORT environment variable for build scripts
    clippy_cmd.env("OPTEE_CLIENT_EXPORT", &config.optee_client_export);

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

fn build_binary(config: &CaBuildConfig) -> Result<()> {
    let component_type = if config.plugin { "Plugin" } else { "CA" };
    println!("Building {} binary...", component_type);

    // Determine target and cross-compile based on arch (CA runs in Normal World Linux)
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, BuildMode::Ca)?;

    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");
    build_cmd.arg("--target").arg(&target);

    // Add --no-default-features if specified
    if config.no_default_features {
        build_cmd.arg("--no-default-features");
    }

    // Add additional features if specified
    if let Some(ref features) = config.features {
        build_cmd.arg("--features").arg(features);
    }

    if !config.debug {
        build_cmd.arg("--release");
    }

    // Configure linker
    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);

    // Set OPTEE_CLIENT_EXPORT environment variable
    build_cmd.env("OPTEE_CLIENT_EXPORT", &config.optee_client_export);

    // Apply custom environment variables
    for (key, value) in &config.env {
        build_cmd.env(key, value);
    }

    // Print the full cargo build command for debugging
    print_cargo_command(&build_cmd, "Building CA binary");

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        print_output_and_bail("build", &build_output)?;
    }

    Ok(())
}

fn post_build(config: &CaBuildConfig) -> Result<PathBuf> {
    if config.plugin {
        copy_plugin(config)
    } else {
        strip_binary(config)
    }
}

fn copy_plugin(config: &CaBuildConfig) -> Result<PathBuf> {
    println!("Processing plugin...");

    // Determine target based on arch (CA runs in Normal World Linux)
    let (target, _cross_compile) = get_target_and_cross_compile(config.arch, BuildMode::Ca)?;

    let profile = if config.debug { "debug" } else { "release" };

    // Use cargo metadata to get the target directory (supports workspace and CARGO_TARGET_DIR)
    let target_directory = get_target_directory_from_metadata()?;
    let target_dir = target_directory.join(target).join(profile);

    // Get the library name from Cargo.toml (we're already in the project directory)
    let lib_name = get_package_name()?;

    // Plugin is built as a shared library (lib<name>.so)
    let plugin_src = common::join_format_and_check::<&str>(
        &target_dir,
        &[],
        &format!("lib{}.so", lib_name),
        "Plugin library",
    )?;

    // Read UUID from specified file
    let uuid = read_uuid_from_file(
        config
            .uuid_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("UUID path is required for plugin builds"))?,
    )?;

    // Copy to <uuid>.plugin.so
    let plugin_dest = target_dir.join(format!("{}.plugin.so", uuid));
    std::fs::copy(&plugin_src, &plugin_dest)?;

    Ok(plugin_dest)
}

fn strip_binary(config: &CaBuildConfig) -> Result<PathBuf> {
    println!("Stripping binary...");

    // Determine target and cross-compile based on arch (CA runs in Normal World Linux)
    let (target, cross_compile) = get_target_and_cross_compile(config.arch, BuildMode::Ca)?;

    let profile = if config.debug { "debug" } else { "release" };

    // Use cargo metadata to get the target directory (supports workspace and CARGO_TARGET_DIR)
    let target_directory = get_target_directory_from_metadata()?;
    let target_dir = target_directory.join(target).join(profile);

    // Get the binary name from Cargo.toml (we're already in the project directory)
    let binary_name = get_package_name()?;

    let binary_path = common::join_and_check(&target_dir, &[binary_name], "Binary")?;

    let objcopy = format!("{}objcopy", cross_compile);

    let strip_output = Command::new(&objcopy)
        .arg("--strip-unneeded")
        .arg(&binary_path)
        .arg(&binary_path) // Strip in place
        .output()?;

    if !strip_output.status.success() {
        print_output_and_bail(&objcopy, &strip_output)?;
    }

    Ok(binary_path)
}
