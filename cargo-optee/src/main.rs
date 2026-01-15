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

use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process;

mod ca_builder;
mod cli;
mod common;
mod config;
mod ta_builder;

use cli::{BuildCommand, Cli, Command, CommonBuildArgs, InstallCommand};

fn main() {
    // Setup cargo environment
    if let Err(e) = setup_cargo_environment() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Drop extra `optee` argument provided by `cargo`.
    let mut found_optee = false;
    let filtered_args: Vec<String> = env::args()
        .filter(|x| {
            if found_optee {
                true
            } else {
                found_optee = x == "optee";
                x != "optee"
            }
        })
        .collect();

    let cli = Cli::parse_from(filtered_args);
    let result = execute_command(cli.cmd);

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn execute_command(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Build(build_cmd) => match build_cmd {
            BuildCommand::TA { build_cmd } => {
                // Convert bool flags to Option<bool>: --std -> Some(true), --no-std -> Some(false), neither -> None
                let std_mode = match (build_cmd.std, build_cmd.no_std) {
                    (true, false) => Some(true),
                    (false, true) => Some(false),
                    _ => None,
                };

                execute_ta_command(
                    build_cmd.common,
                    std_mode,
                    build_cmd.ta_dev_kit_dir,
                    build_cmd.signing_key,
                    build_cmd.uuid_path,
                    None,
                )
            }
            BuildCommand::CA { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                None,
                false,
                None,
            ),
            BuildCommand::Plugin { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                build_cmd.uuid_path,
                true,
                None,
            ),
        },
        Command::Install(install_cmd) => match install_cmd {
            InstallCommand::TA {
                target_dir,
                build_cmd,
            } => {
                // Convert bool flags to Option<bool>: --std -> Some(true), --no-std -> Some(false), neither -> None
                let std_mode = match (build_cmd.std, build_cmd.no_std) {
                    (true, false) => Some(true),
                    (false, true) => Some(false),
                    _ => None,
                };

                execute_ta_command(
                    build_cmd.common,
                    std_mode,
                    build_cmd.ta_dev_kit_dir,
                    build_cmd.signing_key,
                    build_cmd.uuid_path,
                    Some(&target_dir),
                )
            }
            InstallCommand::CA {
                target_dir,
                build_cmd,
            } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                None,
                false,
                Some(&target_dir),
            ),
            InstallCommand::Plugin {
                target_dir,
                build_cmd,
            } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                build_cmd.uuid_path,
                true,
                Some(&target_dir),
            ),
        },
        Command::Clean { clean_cmd } => {
            let project_path = resolve_project_path(clean_cmd.manifest_path.as_ref())?;

            // Clean build artifacts using the common function
            crate::common::clean_project(&project_path)
        }
    }
}

/// Execute TA build or install (shared logic)
fn execute_ta_command(
    common: CommonBuildArgs,
    std: Option<bool>,
    ta_dev_kit_dir: Option<PathBuf>,
    signing_key: Option<PathBuf>,
    uuid_path: Option<PathBuf>,
    install_target_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
    // Resolve project path from manifest or current directory
    let project_path = resolve_project_path(common.manifest_path.as_ref())?;

    // Resolve TA build configuration with priority: CLI > metadata > default
    let ta_config = config::TaBuildConfig::resolve(
        &project_path,
        common.arch,
        Some(common.debug),
        uuid_path,
        common.env,
        common.no_default_features,
        common.features,
        std, // None means read from config, Some(true/false) means CLI override
        ta_dev_kit_dir,
        signing_key,
    )?;

    // Print the final configuration being used
    ta_config.print_config();

    ta_builder::build_ta(ta_config, install_target_dir.map(|p| p.as_path()))
}

/// Execute CA build or install (shared logic)
fn execute_ca_command(
    common: CommonBuildArgs,
    optee_client_export: Option<PathBuf>,
    uuid_path: Option<PathBuf>,
    plugin: bool,
    install_target_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
    // Resolve project path from manifest or current directory
    let project_path = resolve_project_path(common.manifest_path.as_ref())?;

    // Resolve CA build configuration with priority: CLI > metadata > default
    let ca_config = config::CaBuildConfig::resolve(
        &project_path,
        common.arch,
        Some(common.debug),
        uuid_path,
        common.env,
        common.no_default_features,
        common.features,
        optee_client_export,
        plugin,
    )?;

    // Print the final configuration being used
    ca_config.print_config();

    ca_builder::build_ca(ca_config, install_target_dir.map(|p| p.as_path()))
}

/// Resolve project path from manifest path or current directory
fn resolve_project_path(manifest_path: Option<&PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(manifest) = manifest_path {
        let parent = manifest
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?;

        // Normalize: if parent is empty (e.g., manifest is just "Cargo.toml"),
        // use current directory instead
        if parent.as_os_str().is_empty() {
            std::env::current_dir().map_err(Into::into)
        } else {
            // Canonicalize must succeed, otherwise treat as invalid manifest path
            parent
                .canonicalize()
                .map_err(|_| anyhow::anyhow!("Invalid manifest path"))
        }
    } else {
        std::env::current_dir().map_err(Into::into)
    }
}

/// Setup cargo environment by checking availability and adding to PATH if needed
fn setup_cargo_environment() -> anyhow::Result<()> {
    // Check if cargo is already available in PATH
    let cargo_available = std::process::Command::new("which")
        .arg("cargo")
        .output()
        .is_ok_and(|output| output.status.success());

    if cargo_available {
        return Ok(());
    }

    // Check if ~/.cargo/bin/cargo exists
    let cargo_bin_dir = if let Ok(home) = env::var("HOME") {
        let cargo_path = std::path::Path::new(&home)
            .join(".cargo")
            .join("bin")
            .join("cargo");
        if cargo_path.exists() {
            cargo_path.parent().map(|p| p.to_path_buf())
        } else {
            None
        }
    } else {
        None
    };

    // Or check $CARGO_HOME/bin/cargo
    let cargo_bin_dir = cargo_bin_dir.or_else(|| {
        env::var("CARGO_HOME").ok().and_then(|cargo_home| {
            let cargo_path = std::path::Path::new(&cargo_home).join("bin").join("cargo");
            if cargo_path.exists() {
                cargo_path.parent().map(|p| p.to_path_buf())
            } else {
                None
            }
        })
    });

    // If found, add cargo bin directory to current process PATH
    if let Some(cargo_bin_dir) = cargo_bin_dir {
        let current_path = env::var("PATH").unwrap_or_default();
        let separator = if cfg!(windows) { ";" } else { ":" };
        let new_path = format!("{}{}{}", cargo_bin_dir.display(), separator, current_path);
        env::set_var("PATH", &new_path);
        return Ok(());
    }

    // If not found, prompt user to install Cargo
    anyhow::bail!("cargo command not found. Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
}
