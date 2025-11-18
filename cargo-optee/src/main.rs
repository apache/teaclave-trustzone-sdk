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

/// Execute TA build or install (shared logic)
fn execute_ta_command(
    common: CommonBuildArgs,
    std: bool,
    ta_dev_kit_dir: Option<PathBuf>,
    signing_key: Option<PathBuf>,
    uuid_path: Option<PathBuf>,
    install_target_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
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
    let ta_dev_kit_dir = resolve_path_relative_to_project(&ta_dev_kit_dir_config, &project_path);

    // Validate that ta_dev_kit_dir exists (print absolute path)
    if !ta_dev_kit_dir.exists() {
        bail!(
            "TA development kit directory does not exist: {:?}",
            ta_dev_kit_dir
        );
    }

    // Resolve signing key relative to project directory
    let signing_key_config = build_config.resolve_signing_key(&ta_dev_kit_dir_config);
    let signing_key_path = resolve_path_relative_to_project(&signing_key_config, &project_path);

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
    let project_path = if let Some(manifest) = common.manifest_path {
        manifest
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?
            .to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let component_type = if plugin { "plugin" } else { "ca" };

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
        build_cmd: TABuildArgs,
    },
    /// Build a Client Application (Host)
    #[command(about = "Build a Client Application (Host)")]
    CA {
        #[command(flatten)]
        build_cmd: CABuildArgs,
    },
    /// Build a Plugin (Shared Library)
    #[command(about = "Build a Plugin (Shared Library)")]
    Plugin {
        #[command(flatten)]
        build_cmd: PluginBuildArgs,
    },
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build OP-TEE components
    #[clap(name = "build")]
    #[command(subcommand)]
    Build(BuildCommand),
    /// Install OP-TEE components
    #[clap(name = "install")]
    #[command(subcommand)]
    Install(InstallCommand),
    /// Clean OP-TEE components
    #[clap(name = "clean")]
    Clean {
        #[command(flatten)]
        clean_cmd: CleanCommand,
    },
}

#[derive(Debug, Subcommand)]
enum InstallCommand {
    /// Install a Trusted Application (TA)
    #[command(about = "Install a Trusted Application (TA) to target directory")]
    TA {
        /// Target directory to install the TA binary (default: "shared")
        #[arg(long = "target-dir", default_value = "shared")]
        target_dir: PathBuf,

        #[command(flatten)]
        build_cmd: TABuildArgs,
    },
    /// Install a Client Application (Host)
    #[command(about = "Install a Client Application (Host) to target directory")]
    CA {
        /// Target directory to install the CA binary (default: "shared")
        #[arg(long = "target-dir", default_value = "shared")]
        target_dir: PathBuf,

        #[command(flatten)]
        build_cmd: CABuildArgs,
    },
    /// Install a Plugin (Shared Library)
    #[command(about = "Install a Plugin (Shared Library) to target directory")]
    Plugin {
        /// Target directory to install the plugin binary (default: "shared")
        #[arg(long = "target-dir", default_value = "shared")]
        target_dir: PathBuf,

        #[command(flatten)]
        build_cmd: PluginBuildArgs,
    },
}

/// TA-specific build arguments
#[derive(Debug, Args)]
struct TABuildArgs {
    #[command(flatten)]
    common: CommonBuildArgs,

    /// Enable std feature for the TA (default: false)
    #[arg(long = "std")]
    std: bool,

    /// OP-TEE TA development kit export directory
    #[arg(long = "ta-dev-kit-dir")]
    ta_dev_kit_dir: Option<PathBuf>,

    /// TA signing key path (default: TA_DEV_KIT_DIR/keys/default_ta.pem)
    #[arg(long = "signing-key")]
    signing_key: Option<PathBuf>,

    /// UUID file path (default: "../uuid.txt")
    #[arg(long = "uuid-path")]
    uuid_path: Option<PathBuf>,
}

/// CA-specific build arguments
#[derive(Debug, Args)]
struct CABuildArgs {
    #[command(flatten)]
    common: CommonBuildArgs,

    /// OP-TEE client export directory
    #[arg(long = "optee-client-export")]
    optee_client_export: Option<PathBuf>,
}

/// Plugin-specific build arguments
#[derive(Debug, Args)]
struct PluginBuildArgs {
    #[command(flatten)]
    common: CommonBuildArgs,

    /// OP-TEE client export directory
    #[arg(long = "optee-client-export")]
    optee_client_export: Option<PathBuf>,

    /// UUID file path (default: "../uuid.txt")
    #[arg(long = "uuid-path")]
    uuid_path: Option<PathBuf>,
}

/// Clean command arguments
#[derive(Debug, Args)]
struct CleanCommand {
    /// Path to the Cargo.toml manifest file
    #[arg(long = "manifest-path")]
    manifest_path: Option<PathBuf>,
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

/// Setup cargo environment by checking availability and sourcing environment if needed
fn setup_cargo_environment() -> anyhow::Result<()> {
    // Check if cargo is available
    let cargo_available = std::process::Command::new("which")
        .arg("cargo")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

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

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Setup cargo environment
    if let Err(e) = setup_cargo_environment() {
        eprintln!("Error: {}", e);
        process::exit(1);
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
            BuildCommand::TA { build_cmd } => execute_ta_command(
                build_cmd.common,
                build_cmd.std,
                build_cmd.ta_dev_kit_dir,
                build_cmd.signing_key,
                build_cmd.uuid_path,
                None,
            ),
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
            } => execute_ta_command(
                build_cmd.common,
                build_cmd.std,
                build_cmd.ta_dev_kit_dir,
                build_cmd.signing_key,
                build_cmd.uuid_path,
                Some(&target_dir),
            ),
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
            let project_path = if let Some(manifest) = clean_cmd.manifest_path {
                manifest
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?
                    .to_path_buf()
            } else {
                std::env::current_dir()?
            };

            // Clean build artifacts using the common function
            crate::common::clean_project(&project_path)
        }
    }
}
