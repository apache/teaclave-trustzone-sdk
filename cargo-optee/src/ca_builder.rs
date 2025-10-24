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
use std::path::PathBuf;
use std::process::Command;

use crate::common::{
    get_target_and_cross_compile, print_output_and_bail, Arch, ChangeDirectoryGuard,
};

pub struct CaBuildConfig {
    pub arch: Arch,                   // Architecture
    pub optee_client_export: PathBuf, // Path to OP-TEE client export
    pub debug: bool,                  // Debug mode (default false = release)
    pub path: PathBuf,                // Path to CA directory
}

// Main function to build the CA
pub fn build_ca(config: CaBuildConfig) -> Result<()> {
    // Change to the CA directory
    let _guard = ChangeDirectoryGuard::new(&config.path)?;

    println!("Building CA in directory: {:?}", config.path);

    // Step 1: Run clippy for code quality checks
    run_clippy(&config)?;

    // Step 2: Build the CA
    build_binary(&config)?;

    // Step 3: Strip the binary
    strip_binary(&config)?;

    println!("CA build completed successfully!");

    Ok(())
}

fn run_clippy(config: &CaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    // Run cargo fmt
    let fmt_output = Command::new("cargo").arg("fmt").output()?;

    if !fmt_output.status.success() {
        print_output_and_bail("cargo fmt", &fmt_output)?;
    }

    // Determine target based on arch
    let (target, _cross_compile) = get_target_and_cross_compile(config.arch);

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
    println!("Building CA binary...");

    // Determine target and cross-compile based on arch
    let (target, cross_compile) = get_target_and_cross_compile(config.arch);

    let mut build_cmd = Command::new("cargo");
    build_cmd.arg("build");
    build_cmd.arg("--target").arg(&target);

    if !config.debug {
        build_cmd.arg("--release");
    }

    // Configure linker
    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);

    // Set OPTEE_CLIENT_EXPORT environment variable
    build_cmd.env("OPTEE_CLIENT_EXPORT", &config.optee_client_export);

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        print_output_and_bail("build", &build_output)?;
    }

    Ok(())
}

fn strip_binary(config: &CaBuildConfig) -> Result<PathBuf> {
    println!("Stripping binary...");

    // Determine target based on arch
    let (target, cross_compile) = get_target_and_cross_compile(config.arch);

    let profile = if config.debug { "debug" } else { "release" };
    let target_dir = PathBuf::from("target").join(target).join(profile);

    // Get the binary name from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let binary_name = cargo_toml
        .lines()
        .find(|line| line.trim().starts_with("name"))
        .and_then(|line| line.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"'))
        .ok_or_else(|| anyhow::anyhow!("Could not find package name in Cargo.toml"))?;

    let binary_path = target_dir.join(binary_name);

    if !binary_path.exists() {
        bail!("Binary not found at {:?}", binary_path);
    }

    let objcopy = format!("{}objcopy", cross_compile);

    let strip_output = Command::new(&objcopy)
        .arg("--strip-unneeded")
        .arg(&binary_path)
        .arg(&binary_path) // Strip in place
        .output()?;

    if !strip_output.status.success() {
        print_output_and_bail(&objcopy, &strip_output)?;
    }

    println!("CA binary stripped and saved to: {:?}", binary_path);

    Ok(binary_path)
}
