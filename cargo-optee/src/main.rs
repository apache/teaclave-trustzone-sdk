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
use clap::{Args, Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process;

mod ca_builder;
mod common;
mod config;
mod ta_builder;

use common::Arch;

/// Parse environment variable in KEY=VALUE format
fn parse_env_var(s: &str) -> Result<(String, String), String> {
    if let Some(eq_pos) = s.find('=') {
        let (key, value) = s.split_at(eq_pos);
        let value = &value[1..]; // Skip the '=' character
        Ok((key.to_string(), value.to_string()))
    } else {
        Err(format!(
            "Invalid environment variable format: '{}'. Expected 'KEY=VALUE'",
            s
        ))
    }
}

/// Resolve a potentially relative path to an absolute path based on the project directory
fn resolve_path_relative_to_project(path: &PathBuf, project_path: &std::path::Path) -> PathBuf {
    if path.is_absolute() {
        path.clone()
    } else {
        project_path.join(path)
    }
}

/// Common build command arguments shared across TA, CA, and Plugin builds
#[derive(Debug, Args)]
struct CommonBuildArgs {
    /// Path to the Cargo.toml manifest file
    #[arg(long = "manifest-path")]
    manifest_path: Option<PathBuf>,

    /// Target architecture (default: aarch64)
    #[arg(long = "arch")]
    arch: Option<Arch>,

    /// Enable debug build (default: false)
    #[arg(long = "debug")]
    debug: bool,

    /// Environment overrides in the form of `"KEY=VALUE"` strings. This flag can be repeated.
    ///
    /// This is generally not needed to be used explicitly during regular development.
    ///
    /// This makes sense to be used to specify custom var e.g. `RUSTFLAGS`.
    #[arg(long = "env", value_parser = parse_env_var, action = clap::ArgAction::Append)]
    env: Vec<(String, String)>,

    /// Disable default features (will append --no-default-features to cargo build)
    #[arg(long = "no-default-features")]
    no_default_features: bool,

    /// Custom features to enable (will append --features to cargo build)
    #[arg(long = "features")]
    features: Option<String>,
}

#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Build tool for OP-TEE Rust projects")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum BuildCommand {
    /// Build a Trusted Application (TA)
    #[command(about = "Build a Trusted Application (TA)")]
    TA {
        #[command(flatten)]
        common: CommonBuildArgs,

        /// Enable std feature for the TA (default: false)
        ///
        /// It means the customized "optee" target is used for building,
        ///
        /// The builder is "xargo" and enables "--features std"
        #[arg(long = "std")]
        std: bool,

        /// OP-TEE TA development kit export directory, no default, if unset, return error
        #[arg(long = "ta-dev-kit-dir")]
        ta_dev_kit_dir: Option<PathBuf>,

        /// TA signing key path (default: TA_DEV_KIT_DIR/keys/default_ta.pem)
        #[arg(long = "signing-key")]
        signing_key: Option<PathBuf>,

        /// UUID file path (default: "../uuid.txt")
        #[arg(long = "uuid-path")]
        uuid_path: Option<PathBuf>,
    },
    /// Build a Client Application (Host)
    #[command(about = "Build a Client Application (Host)")]
    CA {
        #[command(flatten)]
        common: CommonBuildArgs,

        /// OP-TEE client export directory, no default, if unset, return error
        #[arg(long = "optee-client-export")]
        optee_client_export: Option<PathBuf>,
    },
    /// Build a Plugin (Shared Library)
    #[command(about = "Build a Plugin (Shared Library)")]
    Plugin {
        #[command(flatten)]
        common: CommonBuildArgs,

        /// OP-TEE client export directory, no default, if unset, return error
        #[arg(long = "optee-client-export")]
        optee_client_export: Option<PathBuf>,

        /// UUID file path (default: "../uuid.txt")
        #[arg(long = "uuid-path")]
        uuid_path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build OP-TEE components
    #[clap(name = "build")]
    #[command(subcommand)]
    Build(BuildCommand),
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Setup cargo environment if needed
    if let Ok(home) = env::var("HOME") {
        let cargo_env = format!("{}/.cargo/env", home);
        if std::path::Path::new(&cargo_env).exists() {
            // Add cargo bin to PATH
            let cargo_bin = format!("{}/.cargo/bin", home);
            if let Ok(current_path) = env::var("PATH") {
                let new_path = format!("{}:{}", cargo_bin, current_path);
                env::set_var("PATH", new_path);
            }
        } else {
            eprintln!("Error: Cargo environment file not found at: {}. Please ensure Rust and Cargo are installed.", cargo_env);
            process::exit(1);
        }
    }

    let cli = Cli::parse();
    let result = execute_command(cli.cmd);

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn execute_command(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Build(build_cmd) => match build_cmd {
            BuildCommand::TA {
                common,
                std,
                ta_dev_kit_dir,
                signing_key,
                uuid_path,
            } => {
                // Resolve project path from manifest or current directory
                let project_path = if let Some(manifest) = common.manifest_path {
                    manifest
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?
                        .to_path_buf()
                } else {
                    std::env::current_dir()?
                };

                // Resolve build configuration with priority: CLI > metadata > error
                let build_config = config::BuildConfig::resolve(
                    &project_path,
                    "ta", // Component type for TA
                    common.arch,
                    Some(common.debug),
                    Some(std),
                    ta_dev_kit_dir,
                    None, // optee_client_export not needed for TA
                    signing_key,
                    uuid_path.clone(),
                )?;

                // Print the final configuration being used
                build_config.print_config("ta", &project_path);

                // Get required ta_dev_kit_dir and resolve relative to project
                let ta_dev_kit_dir_config = build_config.require_ta_dev_kit_dir()?;
                let ta_dev_kit_dir =
                    resolve_path_relative_to_project(&ta_dev_kit_dir_config, &project_path);

                // Validate that ta_dev_kit_dir exists (print absolute path)
                if !ta_dev_kit_dir.exists() {
                    bail!(
                        "TA development kit directory does not exist: {:?}",
                        ta_dev_kit_dir
                    );
                }

                // Resolve signing key relative to project directory
                let signing_key_config = build_config.resolve_signing_key(&ta_dev_kit_dir_config);
                let signing_key_path =
                    resolve_path_relative_to_project(&signing_key_config, &project_path);

                // Validate that signing key exists (print absolute path)
                if !signing_key_path.exists() {
                    bail!("Signing key file does not exist: {:?}", signing_key_path);
                }

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

                ta_builder::build_ta(ta_builder::TaBuildConfig {
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
                })
            }
            BuildCommand::CA {
                common,
                optee_client_export,
            } => {
                // Resolve project path from manifest or current directory
                let project_path = if let Some(manifest) = common.manifest_path {
                    manifest
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?
                        .to_path_buf()
                } else {
                    std::env::current_dir()?
                };

                // Resolve build configuration
                let build_config = config::BuildConfig::resolve(
                    &project_path,
                    "ca", // Component type for CA
                    common.arch,
                    Some(common.debug),
                    None, // std not applicable for CA
                    None, // ta_dev_kit_dir not needed for CA
                    optee_client_export,
                    None, // signing_key not needed for CA
                    None, // uuid_path not needed for CA
                )?;

                // Print the final configuration being used
                build_config.print_config("ca", &project_path);

                // Get required optee_client_export and resolve relative to project
                let optee_client_export_config = build_config.require_optee_client_export()?;
                let optee_client_export =
                    resolve_path_relative_to_project(&optee_client_export_config, &project_path);

                // Validate that optee_client_export exists (print absolute path)
                if !optee_client_export.exists() {
                    bail!(
                        "OP-TEE client export directory does not exist: {:?}",
                        optee_client_export
                    );
                }

                // Merge env variables: CLI overrides + metadata env
                let mut merged_env = build_config.env.clone();
                merged_env.extend(common.env);

                ca_builder::build_ca(ca_builder::CaBuildConfig {
                    arch: build_config.arch,
                    optee_client_export,
                    debug: build_config.debug,
                    path: project_path,
                    plugin: false,
                    uuid_path: None, // Not needed for regular CA
                    env: merged_env,
                    no_default_features: common.no_default_features,
                    features: common.features,
                })
            }
            BuildCommand::Plugin {
                common,
                optee_client_export,
                uuid_path,
            } => {
                // Resolve project path from manifest or current directory
                let project_path = if let Some(manifest) = common.manifest_path {
                    manifest
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?
                        .to_path_buf()
                } else {
                    std::env::current_dir()?
                };

                // Resolve build configuration
                let build_config = config::BuildConfig::resolve(
                    &project_path,
                    "plugin", // Component type for Plugin
                    common.arch,
                    Some(common.debug),
                    None,                // std not applicable for plugin
                    None,                // ta_dev_kit_dir not needed for plugin (runs on host)
                    optee_client_export, // Plugin needs optee_client_export (runs on host)
                    None,                // signing_key not needed for plugin
                    uuid_path,
                )?;

                // Print the final configuration being used
                build_config.print_config("plugin", &project_path);

                // Get required optee_client_export and resolve relative to project
                let optee_client_export_config = build_config.require_optee_client_export()?;
                let optee_client_export =
                    resolve_path_relative_to_project(&optee_client_export_config, &project_path);

                // Validate that optee_client_export exists (print absolute path)
                if !optee_client_export.exists() {
                    bail!(
                        "OP-TEE client export directory does not exist: {:?}",
                        optee_client_export
                    );
                }

                // Merge env variables: CLI overrides + metadata env
                let mut merged_env = build_config.env.clone();
                merged_env.extend(common.env);

                ca_builder::build_ca(ca_builder::CaBuildConfig {
                    arch: build_config.arch,
                    optee_client_export,
                    debug: build_config.debug,
                    path: project_path,
                    plugin: true,
                    uuid_path: build_config.uuid_path.clone(),
                    env: merged_env,
                    no_default_features: common.no_default_features,
                    features: common.features,
                })
            }
        },
    }
}
