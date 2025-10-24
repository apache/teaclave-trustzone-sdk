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

use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process::abort;

mod ca_builder;
mod common;
mod ta_builder;

use common::Arch;

#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Build tool for OP-TEE Rust projects")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
struct BuildTypeCommonOptions {
    /// Path to the app directory (default: current directory)
    #[arg(long = "path", default_value = ".")]
    path: PathBuf,

    /// Target architecture: aarch64 or arm (default: aarch64)
    #[arg(long = "arch", default_value = "aarch64")]
    arch: Arch,

    /// Path to the UUID file (default: ../uuid.txt)
    #[arg(long = "uuid_path", default_value = "../uuid.txt")]
    uuid_path: PathBuf,

    /// Build in debug mode (default is release)
    #[arg(long = "debug")]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum BuildCommand {
    /// Build a Trusted Application
    TA {
        /// Enable std feature for the TA
        #[arg(long = "std")]
        std: bool,

        /// Path to the TA dev kit directory (mandatory)
        #[arg(long = "ta_dev_kit_dir", required = true)]
        ta_dev_kit_dir: PathBuf,

        /// Path to the TA signing key (default: $(TA_DEV_KIT_DIR)/keys/default_ta.pem)
        #[arg(long = "signing_key")]
        signing_key: Option<PathBuf>,

        #[command(flatten)]
        common: BuildTypeCommonOptions,
    },
    /// Build a Client Application (Host)
    CA {
        /// Path to the OP-TEE client export directory (mandatory)
        #[arg(long = "optee_client_export", required = true)]
        optee_client_export: PathBuf,

        #[command(flatten)]
        common: BuildTypeCommonOptions,
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
            abort();
        }
    }

    let cli = Cli::parse();
    let result = execute_command(cli.cmd);

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        abort();
    }
}

fn execute_command(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Build(build_cmd) => match build_cmd {
            BuildCommand::TA {
                std,
                ta_dev_kit_dir,
                signing_key,
                common,
            } => {
                // Determine signing key path
                let signing_key_path = signing_key
                    .unwrap_or_else(|| ta_dev_kit_dir.join("keys").join("default_ta.pem"));

                ta_builder::build_ta(ta_builder::TaBuildConfig {
                    arch: common.arch,
                    std,
                    ta_dev_kit_dir,
                    signing_key: signing_key_path,
                    uuid_path: common.uuid_path,
                    debug: common.debug,
                    path: common.path,
                })
            }
            BuildCommand::CA {
                optee_client_export,
                common,
            } => ca_builder::build_ca(ca_builder::CaBuildConfig {
                arch: common.arch,
                optee_client_export,
                debug: common.debug,
                path: common.path,
            }),
        },
    }
}
