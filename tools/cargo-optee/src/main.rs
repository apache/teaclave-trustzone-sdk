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

use cargo_optee::ca_builder;
use cargo_optee::cli::{
    BuildCommand, Cli, ClippyCommand, Command, CommonBuildArgs, InstallCommand, UuidKind,
};
use cargo_optee::common::absolute_from_cwd;
use cargo_optee::config;
use cargo_optee::elf_uuid;
use cargo_optee::ta_builder;
use clap::Parser;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

fn main() {
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
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn execute_command(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Build(build_cmd) => match build_cmd {
            BuildCommand::TA { build_cmd } => {
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
                    None,
                    false,
                )
            }
            BuildCommand::CA { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                false,
                None,
                false,
            ),
            BuildCommand::Plugin { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                true,
                None,
                false,
            ),
        },
        Command::Install(install_cmd) => match install_cmd {
            InstallCommand::TA {
                target_dir,
                build_cmd,
            } => {
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
                    Some(absolute_from_cwd(&target_dir)?),
                    false,
                )
            }
            InstallCommand::CA {
                target_dir,
                build_cmd,
            } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                false,
                Some(absolute_from_cwd(&target_dir)?),
                false,
            ),
            InstallCommand::Plugin {
                target_dir,
                build_cmd,
            } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                true,
                Some(absolute_from_cwd(&target_dir)?),
                false,
            ),
        },
        Command::Clippy(clippy_cmd) => match clippy_cmd {
            ClippyCommand::TA { build_cmd } => {
                let std_mode = match (build_cmd.std, build_cmd.no_std) {
                    (true, false) => Some(true),
                    (false, true) => Some(false),
                    _ => None,
                };

                execute_ta_command(
                    build_cmd.common,
                    std_mode,
                    build_cmd.ta_dev_kit_dir,
                    None,
                    None,
                    true,
                )
            }
            ClippyCommand::CA { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                false,
                None,
                true,
            ),
            ClippyCommand::Plugin { build_cmd } => execute_ca_command(
                build_cmd.common,
                build_cmd.optee_client_export,
                true,
                None,
                true,
            ),
        },
        Command::Clean { clean_cmd } => {
            let project_path = resolve_project_path(clean_cmd.manifest_path.as_ref())?;
            cargo_optee::common::clean_project(&project_path)
        }
        Command::InspectUuid { kind, elf } => inspect_uuid(kind, &elf),
    }
}

fn inspect_uuid(kind: UuidKind, elf: &Path) -> anyhow::Result<()> {
    let uuid = match kind {
        UuidKind::Ta => elf_uuid::read_ta_uuid(elf)?,
        UuidKind::Plugin => elf_uuid::read_plugin_uuid(elf)?,
    };
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{uuid}")?;
    Ok(())
}

/// Execute TA build or install (shared logic)
fn execute_ta_command(
    common: CommonBuildArgs,
    std: Option<bool>,
    ta_dev_kit_dir: Option<PathBuf>,
    signing_key: Option<PathBuf>,
    install_target_dir: Option<PathBuf>,
    lint_only: bool,
) -> anyhow::Result<()> {
    let project_path = resolve_project_path(common.manifest_path.as_ref())?;

    let ta_config = config::TaBuildConfig::resolve(
        &project_path,
        config::BuildOverrides {
            arch: common.arch,
            debug: cli_debug_override(common.debug),
            env: common.env,
            no_default_features: common.no_default_features,
            features: common.features,
        },
        std,
        ta_dev_kit_dir,
        signing_key,
        !lint_only,
    )?;

    if lint_only {
        ta_config.print_lint_config();
        ta_builder::clippy_ta(ta_config)
    } else {
        ta_config.print_config();
        ta_builder::build_ta(ta_config, install_target_dir.as_deref())
    }
}

/// Execute CA build or install (shared logic)
fn execute_ca_command(
    common: CommonBuildArgs,
    optee_client_export: Option<PathBuf>,
    plugin: bool,
    install_target_dir: Option<PathBuf>,
    lint_only: bool,
) -> anyhow::Result<()> {
    let project_path = resolve_project_path(common.manifest_path.as_ref())?;

    let ca_config = config::CaBuildConfig::resolve(
        &project_path,
        config::BuildOverrides {
            arch: common.arch,
            debug: cli_debug_override(common.debug),
            env: common.env,
            no_default_features: common.no_default_features,
            features: common.features,
        },
        optee_client_export,
        plugin,
    )?;

    if lint_only {
        ca_config.print_lint_config();
        ca_builder::clippy_ca(ca_config)
    } else {
        ca_config.print_config();
        ca_builder::build_ca(ca_config, install_target_dir.as_deref())
    }
}

/// `--debug` means force debug. Omitting it leaves metadata/default in place.
fn cli_debug_override(debug: bool) -> Option<bool> {
    debug.then_some(true)
}

/// Resolve project path from manifest path or current directory
fn resolve_project_path(manifest_path: Option<&PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(manifest) = manifest_path {
        let parent = manifest
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid manifest path"))?;

        if parent.as_os_str().is_empty() {
            std::env::current_dir().map_err(Into::into)
        } else {
            parent
                .canonicalize()
                .map_err(|_| anyhow::anyhow!("Invalid manifest path"))
        }
    } else {
        std::env::current_dir().map_err(Into::into)
    }
}
