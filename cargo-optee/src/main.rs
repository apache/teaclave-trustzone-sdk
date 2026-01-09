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

use anyhow::bail;
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
use config::ComponentType;

/// Path type for validation
enum PathType {
    /// Expects a directory
    Directory,
    /// Expects a file
    File,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

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
                let std = if build_cmd.std {
                    Some(true)
                } else if build_cmd.no_std {
                    Some(false)
                } else {
                    None
                };
                execute_ta_command(
                    build_cmd.common,
                    std,
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

    // Resolve build configuration with priority: CLI > metadata > error
    let build_config = config::BuildConfig::resolve(
        &project_path,
        ComponentType::Ta,
        common.arch,
        Some(common.debug),
        std, // None means read from config, Some(true/false) means CLI override
        ta_dev_kit_dir,
        None, // optee_client_export not needed for TA
        signing_key,
        uuid_path.clone(),
    )?;

    // Print the final configuration being used
    build_config.print_config(ComponentType::Ta, &project_path);

    // Get required ta_dev_kit_dir and resolve relative to project
    let ta_dev_kit_dir_config = build_config.require_ta_dev_kit_dir()?;
    let ta_dev_kit_dir = resolve_path_relative_to_project(
        &ta_dev_kit_dir_config,
        &project_path,
        PathType::Directory,
        "TA development kit directory",
    )?;

    // Resolve signing key relative to project directory
    let signing_key_config = build_config.resolve_signing_key(&ta_dev_kit_dir_config);
    let signing_key_path = resolve_path_relative_to_project(
        &signing_key_config,
        &project_path,
        PathType::File,
        "Signing key file",
    )?;

    // Resolve UUID path: if provided via CLI, it's relative to current dir
    // if from metadata, it's relative to project dir
    let resolved_uuid_path = if uuid_path.is_some() {
        // CLI provided - resolve relative to current directory
        std::env::current_dir()?.join(build_config.get_uuid_path())
    } else {
        // From metadata or default - resolve relative to project directory
        project_path.join(build_config.get_uuid_path())
    };

    // Merge env variables: CLI overrides + metadata env
    let mut merged_env = build_config.env.clone();
    merged_env.extend(common.env);

    let ta_config = ta_builder::TaBuildConfig {
        arch: build_config.arch,
        std: build_config.std,
        ta_dev_kit_dir,
        signing_key: signing_key_path,
        debug: build_config.debug,
        path: project_path,
        uuid_path: resolved_uuid_path,
        env: merged_env,
        no_default_features: common.no_default_features,
        features: common.features,
    };

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

    let component_type = if plugin {
        ComponentType::Plugin
    } else {
        ComponentType::Ca
    };

    // Resolve build configuration
    let build_config = config::BuildConfig::resolve(
        &project_path,
        component_type,
        common.arch,
        Some(common.debug),
        None, // std not applicable for CA/Plugin
        None, // ta_dev_kit_dir not needed for CA/Plugin
        optee_client_export,
        None, // signing_key not needed for CA/Plugin
        uuid_path,
    )?;

    // Print the final configuration being used
    build_config.print_config(component_type, &project_path);

    // Get required optee_client_export and resolve relative to project
    let optee_client_export_config = build_config.require_optee_client_export()?;
    let optee_client_export = resolve_path_relative_to_project(
        &optee_client_export_config,
        &project_path,
        PathType::Directory,
        "OP-TEE client export directory",
    )?;

    // Merge env variables: CLI overrides + metadata env
    let mut merged_env = build_config.env.clone();
    merged_env.extend(common.env);

    let ca_config = ca_builder::CaBuildConfig {
        arch: build_config.arch,
        optee_client_export,
        debug: build_config.debug,
        path: project_path,
        plugin,
        uuid_path: if plugin {
            build_config.uuid_path.clone()
        } else {
            None
        },
        env: merged_env,
        no_default_features: common.no_default_features,
        features: common.features,
    };

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

/// Resolve a potentially relative path to an absolute path based on the project directory
/// and validate that it exists
fn resolve_path_relative_to_project(
    path: &PathBuf,
    project_path: &std::path::Path,
    path_type: PathType,
    error_context: &str,
) -> anyhow::Result<PathBuf> {
    let resolved_path = if path.is_absolute() {
        path.clone()
    } else {
        project_path.join(path)
    };

    // Validate that the path exists
    if !resolved_path.exists() {
        bail!("{} does not exist: {:?}", error_context, resolved_path);
    }

    // Additional validation: check if it's actually a directory or file as expected
    match path_type {
        PathType::Directory => {
            if !resolved_path.is_dir() {
                bail!("{} is not a directory: {:?}", error_context, resolved_path);
            }
        }
        PathType::File => {
            if !resolved_path.is_file() {
                bail!("{} is not a file: {:?}", error_context, resolved_path);
            }
        }
    }

    Ok(resolved_path)
}

/// Setup cargo environment by checking availability and sourcing environment if needed
fn setup_cargo_environment() -> anyhow::Result<()> {
    // Check if cargo is available
    let cargo_available = std::process::Command::new("which")
        .arg("cargo")
        .output()
        .is_ok_and(|output| output.status.success());

    if cargo_available {
        return Ok(());
    }

    // Try to source .cargo/env from ~/.cargo/env or $CARGO_HOME/env
    let mut sourced = false;
    if let Ok(home) = env::var("HOME") {
        sourced = source_cargo_env(&format!("{}/.cargo/env", home));
    }
    if !sourced {
        if let Ok(cargo_home) = env::var("CARGO_HOME") {
            sourced = source_cargo_env(&format!("{}/env", cargo_home));
        }
    }

    if !sourced {
        anyhow::bail!("cargo command not found. Please install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
    }

    Ok(())
}

/// Source cargo environment from a given path
fn source_cargo_env(env_path: &str) -> bool {
    if std::path::Path::new(env_path).exists() {
        std::process::Command::new("bash")
            .arg("-c")
            .arg(format!("source {}", env_path))
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    } else {
        false
    }
}
