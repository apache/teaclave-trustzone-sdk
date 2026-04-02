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

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::common::Arch;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
pub enum BuildCommand {
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
pub enum InstallCommand {
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

/// Clean command arguments
#[derive(Debug, Args)]
pub struct CleanCommand {
    /// Path to the Cargo.toml manifest file
    #[arg(long = "manifest-path")]
    pub manifest_path: Option<PathBuf>,
}

/// Common build command arguments shared across TA, CA, and Plugin builds
#[derive(Debug, Args)]
pub struct CommonBuildArgs {
    /// Path to the Cargo.toml manifest file
    #[arg(long = "manifest-path")]
    pub manifest_path: Option<PathBuf>,

    /// Target architecture (default: aarch64)
    #[arg(long = "arch")]
    pub arch: Option<Arch>,

    /// Enable debug build (default: false)
    #[arg(long = "debug")]
    pub debug: bool,

    /// Environment overrides in the form of `"KEY=VALUE"` strings. This flag can be repeated.
    ///
    /// This is generally not needed to be used explicitly during regular development.
    ///
    /// This makes sense to be used to specify custom var e.g. `RUSTFLAGS`.
    #[arg(long = "env", value_parser = parse_env_var, action = clap::ArgAction::Append)]
    pub env: Vec<(String, String)>,

    /// Disable default features (will append --no-default-features to cargo build)
    #[arg(long = "no-default-features")]
    pub no_default_features: bool,

    /// Custom features to enable (will append --features to cargo build)
    #[arg(long = "features")]
    pub features: Option<String>,
}

/// TA-specific build arguments
#[derive(Debug, Args)]
pub struct TABuildArgs {
    #[command(flatten)]
    pub common: CommonBuildArgs,

    /// Enable std feature for the TA
    /// If neither --std nor --no-std is specified, the value will be read from Cargo.toml metadata
    #[arg(long = "std", action = clap::ArgAction::SetTrue, conflicts_with = "no_std")]
    pub std: bool,

    /// Disable std feature for the TA (use no-std mode)
    /// If neither --std nor --no-std is specified, the value will be read from Cargo.toml metadata
    #[arg(long = "no-std", action = clap::ArgAction::SetTrue, conflicts_with = "std")]
    pub no_std: bool,

    /// OP-TEE TA development kit export directory
    #[arg(long = "ta-dev-kit-dir")]
    pub ta_dev_kit_dir: Option<PathBuf>,

    /// TA signing key path (default: TA_DEV_KIT_DIR/keys/default_ta.pem)
    #[arg(long = "signing-key")]
    pub signing_key: Option<PathBuf>,

    /// UUID file path (default: "../uuid.txt")
    #[arg(long = "uuid-path")]
    pub uuid_path: Option<PathBuf>,
}

/// CA-specific build arguments
#[derive(Debug, Args)]
pub struct CABuildArgs {
    #[command(flatten)]
    pub common: CommonBuildArgs,

    /// OP-TEE client export directory
    #[arg(long = "optee-client-export")]
    pub optee_client_export: Option<PathBuf>,
}

/// Plugin-specific build arguments
#[derive(Debug, Args)]
pub struct PluginBuildArgs {
    #[command(flatten)]
    pub common: CommonBuildArgs,

    /// OP-TEE client export directory
    #[arg(long = "optee-client-export")]
    pub optee_client_export: Option<PathBuf>,

    /// UUID file path (default: "../uuid.txt")
    #[arg(long = "uuid-path")]
    pub uuid_path: Option<PathBuf>,
}

/// Parse environment variable in KEY=VALUE format
pub fn parse_env_var(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .ok_or_else(|| {
            format!(
                "Invalid environment variable format: '{}'. Expected 'KEY=VALUE'",
                s
            )
        })
}
