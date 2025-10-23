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
use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process::abort;

mod ta_builder;

#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Build tool for OP-TEE Rust projects")]
pub(crate) struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Build a Trusted Application (TA)
    #[clap(name = "build")]
    Build {
        /// Type of build target (currently only 'ta' is supported)
        #[arg(value_name = "TYPE")]
        build_type: String,

        /// Path to the TA directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: PathBuf,

        /// Target architecture: aarch64 or arm (default: aarch64)
        #[arg(long = "arch", default_value = "aarch64")]
        arch: String,

        /// Enable std feature for the TA
        #[arg(long = "std")]
        std: bool,

        /// Path to the TA dev kit directory (mandatory)
        #[arg(long = "ta_dev_kit_dir", required = true)]
        ta_dev_kit_dir: PathBuf,

        /// Path to the TA signing key (default: $(TA_DEV_KIT_DIR)/keys/default_ta.pem)
        #[arg(long = "signing_key")]
        signing_key: Option<PathBuf>,

        /// Path to the UUID file (default: ../uuid.txt)
        #[arg(long = "uuid_path", default_value = "../uuid.txt")]
        uuid_path: PathBuf,

        /// Build in debug mode (default is release)
        #[arg(long = "debug")]
        debug: bool,
    },
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
        Command::Build {
            build_type,
            path,
            arch,
            std,
            ta_dev_kit_dir,
            signing_key,
            uuid_path,
            debug,
        } => {
            // Validate build type
            if build_type != "ta" {
                bail!(
                    "Invalid build type '{}'. Only 'ta' is supported.",
                    build_type
                );
            }

            // Determine signing key path
            let signing_key_path =
                signing_key.unwrap_or_else(|| ta_dev_kit_dir.join("keys").join("default_ta.pem"));

            ta_builder::build_ta(ta_builder::TaBuildConfig {
                arch,
                std,
                ta_dev_kit_dir,
                signing_key: signing_key_path,
                uuid_path,
                debug,
                path,
            })
        }
    }
}
