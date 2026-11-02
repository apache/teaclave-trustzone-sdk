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

use crate::common::{
    BuildMode, CargoArtifactKind, absolute_dir, apply_clippy_deny_lints, cargo_command_in,
    manifest_path, print_cargo_command, publish_tempfile, resolve_target_and_cross_compile,
    run_cargo_build, run_streaming_command, unique_cargo_artifact, unique_tempfile_in,
};
use crate::config::CaBuildConfig;
use crate::elf_uuid;

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

// Main function to build the CA, optionally installing to a target directory
pub fn build_ca(mut config: CaBuildConfig, install_dir: Option<&Path>) -> Result<()> {
    config.path = absolute_dir(&config.path)?;

    let component_type = if config.plugin { "Plugin" } else { "CA" };
    println!(
        "Building {} in directory: {}",
        component_type,
        config.path.display()
    );

    let built_path = build_binary(&config)?;

    // Step 3: Post-build processing (strip for binaries, copy for plugins)
    let final_binary = post_build(&config, &built_path)?;

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

        let dest_name = final_binary
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Could not get binary name"))?;
        let dest_path = install_dir.join(dest_name);
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

/// Run cargo fmt and clippy with the same target/env as a CA/plugin build.
pub fn clippy_ca(mut config: CaBuildConfig) -> Result<()> {
    config.path = absolute_dir(&config.path)?;
    run_clippy(&config)
}

fn run_clippy(config: &CaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    let mut fmt_cmd = cargo_command_in(&config.path);
    fmt_cmd
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path(&config.path));
    run_streaming_command(&mut fmt_cmd, "cargo fmt")?;

    let mut clippy_cmd = setup_ca_cargo_command(config, "clippy")?;
    apply_clippy_deny_lints(&mut clippy_cmd);
    run_streaming_command(&mut clippy_cmd, "cargo clippy")?;

    Ok(())
}

fn setup_ca_cargo_command(config: &CaBuildConfig, command: &str) -> Result<std::process::Command> {
    let (target, _cross_compile) = resolve_target_and_cross_compile(config.arch, BuildMode::Ca)?;
    let mut cmd = cargo_command_in(&config.path);
    cmd.arg(command);
    cmd.arg("--manifest-path").arg(manifest_path(&config.path));
    cmd.arg("--target").arg(&target);
    if config.no_default_features {
        cmd.arg("--no-default-features");
    }
    if let Some(ref features) = config.features {
        cmd.arg("--features").arg(features);
    }
    cmd.env("OPTEE_CLIENT_EXPORT", &config.optee_client_export);
    for (key, value) in &config.env {
        cmd.env(key, value);
    }
    Ok(cmd)
}

fn build_binary(config: &CaBuildConfig) -> Result<PathBuf> {
    let component_type = if config.plugin { "Plugin" } else { "CA" };
    println!("Building {} binary...", component_type);

    let (target, cross_compile) = resolve_target_and_cross_compile(config.arch, BuildMode::Ca)?;

    let mut build_cmd = setup_ca_cargo_command(config, "build")?;
    if !config.debug {
        build_cmd.arg("--release");
    }

    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);
    build_cmd.arg("--message-format=json");

    print_cargo_command(&build_cmd, "Building CA binary");

    let kind = if config.plugin {
        CargoArtifactKind::Cdylib
    } else {
        CargoArtifactKind::Bin
    };
    let manifest = manifest_path(&config.path);
    let artifacts = run_cargo_build(&mut build_cmd, &manifest, kind)?;

    unique_cargo_artifact(artifacts, &manifest, kind)
}

fn post_build(config: &CaBuildConfig, built_path: &Path) -> Result<PathBuf> {
    if config.plugin {
        copy_plugin(built_path)
    } else {
        strip_binary(config, built_path)
    }
}

fn copy_plugin(plugin_src: &Path) -> Result<PathBuf> {
    println!("Processing plugin...");

    let uuid = elf_uuid::read_plugin_uuid(plugin_src)?;
    let plugin_dest = plugin_src.with_file_name(format!("{}.plugin.so", uuid));
    let parent = plugin_src
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Plugin path has no parent: {:?}", plugin_src))?;
    let tmp = unique_tempfile_in(parent, &format!("{}.plugin.so.", uuid), "")?;
    std::fs::copy(plugin_src, tmp.path())?;
    publish_tempfile(tmp, &plugin_dest)?;

    Ok(plugin_dest)
}

fn strip_binary(config: &CaBuildConfig, binary_path: &Path) -> Result<PathBuf> {
    println!("Stripping binary...");

    let (_, cross_compile) = resolve_target_and_cross_compile(config.arch, BuildMode::Ca)?;
    let objcopy = format!("{}objcopy", cross_compile);

    let mut strip_cmd = std::process::Command::new(&objcopy);
    strip_cmd
        .arg("--strip-unneeded")
        .arg(binary_path)
        .arg(binary_path); // Strip in place
    run_streaming_command(&mut strip_cmd, &objcopy)?;

    Ok(binary_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Arch;

    fn ca_config() -> CaBuildConfig {
        CaBuildConfig {
            arch: Arch::Aarch64,
            debug: false,
            path: PathBuf::from("/tmp/project"),
            env: vec![("CUSTOM".into(), "1".into())],
            no_default_features: true,
            features: Some("needed".into()),
            optee_client_export: PathBuf::from("/tmp/client-export"),
            plugin: false,
        }
    }

    /// Guarantee: `clippy` and `build` receive the same `--features`,
    /// `--no-default-features`, and custom env, so clippy lints the crate
    /// graph that will be signed and installed.
    #[test]
    fn clippy_and_build_share_package_flags() {
        let config = ca_config();
        let clippy = setup_ca_cargo_command(&config, "clippy").unwrap();
        let build = setup_ca_cargo_command(&config, "build").unwrap();
        for cmd in [&clippy, &build] {
            let args: Vec<_> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            assert!(args.contains(&"--features".to_string()));
            assert!(args.contains(&"needed".to_string()));
            assert!(args.contains(&"--no-default-features".to_string()));
            let env = cmd
                .get_envs()
                .find(|(k, _)| k.to_str() == Some("CUSTOM"))
                .and_then(|(_, v)| v)
                .map(|v| v.to_string_lossy().into_owned());
            assert_eq!(env.as_deref(), Some("1"));
        }
    }
}
