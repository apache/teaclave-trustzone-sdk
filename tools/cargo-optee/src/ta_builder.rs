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

use crate::common;
use crate::common::{
    BuildArtifact, BuildMode, CargoArtifactKind, absolute_dir, apply_clippy_deny_lints,
    cargo_command_in, manifest_path, print_cargo_command, print_output_and_bail, publish_tempfile,
    resolve_target_and_cross_compile, set_ta_file_permissions, unique_cargo_artifact,
    unique_tempfile_in,
};
use crate::config::TaBuildConfig;
use crate::elf_uuid;

use anyhow::{Result, bail};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// Embed the target JSON files at compile time
const AARCH64_TARGET_JSON: &str = include_str!("../aarch64-unknown-optee.json");
const ARM_TARGET_JSON: &str = include_str!("../arm-unknown-optee.json");

// Main function to build the TA, optionally installing to a target directory
pub fn build_ta(mut config: TaBuildConfig, install_dir: Option<&Path>) -> Result<()> {
    config.path = absolute_dir(&config.path)?;
    let manifest = manifest_path(&config.path);
    if !manifest.exists() {
        bail!(
            "No Cargo.toml found in TA project directory: {:?}\n\
            Please run cargo-optee from a TA project directory or specify --manifest-path",
            config.path
        );
    }

    // Check if required cross-compile toolchain is available
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (_, cross_compile_prefix) = resolve_target_and_cross_compile(config.arch, build_mode)?;
    check_toolchain_exists(&cross_compile_prefix)?;

    println!("Building TA in directory: {}", config.path.display());

    let elf_path = build_binary(&config)?;
    let stripped = strip_binary(&config, &elf_path)?;
    let artifact = sign_ta(&config, stripped.path())?;

    // Step 5: Install if requested
    if let Some(install_dir) = install_dir {
        // Check if install directory exists
        if !install_dir.exists() {
            bail!("Install directory does not exist: {:?}", install_dir);
        }

        let dest_path = install_dir.join(format!("{}.ta", artifact.uuid));
        fs::copy(&artifact.path, &dest_path)?;
        set_ta_file_permissions(&dest_path)?;

        println!(
            "TA installed to: {:?}",
            dest_path.canonicalize().unwrap_or(dest_path)
        );
    }

    println!("TA build successfully!");

    Ok(())
}

/// Run cargo fmt and clippy with the same target/env as a TA build. Does not
/// strip or sign, so a signing key is not required.
pub fn clippy_ta(mut config: TaBuildConfig) -> Result<()> {
    config.path = absolute_dir(&config.path)?;
    let manifest = manifest_path(&config.path);
    if !manifest.exists() {
        bail!(
            "No Cargo.toml found in TA project directory: {:?}\n\
            Please run cargo-optee from a TA project directory or specify --manifest-path",
            config.path
        );
    }

    run_clippy(&config)
}

fn run_clippy(config: &TaBuildConfig) -> Result<()> {
    println!("Running cargo fmt and clippy...");

    let fmt_output = cargo_command_in(&config.path)
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path(&config.path))
        .output()?;

    if !fmt_output.status.success() {
        print_output_and_bail("cargo fmt", &fmt_output)?;
    }

    // Setup clippy command with common environment
    let (mut clippy_cmd, _temp_dir) = setup_build_command(config, "clippy")?;
    apply_clippy_deny_lints(&mut clippy_cmd);

    let clippy_output = clippy_cmd.output()?;

    if !clippy_output.status.success() {
        print_output_and_bail("clippy", &clippy_output)?;
    }

    Ok(())
}

fn build_binary(config: &TaBuildConfig) -> Result<PathBuf> {
    // Determine target and cross-compile based on arch and std mode
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (target, cross_compile) = resolve_target_and_cross_compile(config.arch, build_mode)?;

    let (mut build_cmd, _temp_dir) = setup_build_command(config, "build")?;

    if !config.debug {
        build_cmd.arg("--release");
    }

    // Configure linker
    let linker = format!("{}gcc", cross_compile);
    let linker_cfg = format!("target.{}.linker=\"{}\"", target, linker);
    build_cmd.arg("--config").arg(&linker_cfg);
    build_cmd.arg("--message-format=json");

    // Print the full cargo build command for debugging
    print_cargo_command(&build_cmd, "Building TA binary");

    let build_output = build_cmd.output()?;

    if !build_output.status.success() {
        let _ = unique_cargo_artifact(
            &build_output.stdout,
            &manifest_path(&config.path),
            CargoArtifactKind::Bin,
        );
        print_output_and_bail("build", &build_output)?;
    }

    unique_cargo_artifact(
        &build_output.stdout,
        &manifest_path(&config.path),
        CargoArtifactKind::Bin,
    )
}

fn strip_binary(config: &TaBuildConfig, binary_path: &Path) -> Result<tempfile::NamedTempFile> {
    println!("Stripping binary...");

    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (_, cross_compile) = resolve_target_and_cross_compile(config.arch, build_mode)?;

    let file_name = binary_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Binary path has no file name: {:?}", binary_path))?;
    let parent = binary_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Binary path has no parent: {:?}", binary_path))?;
    let stripped = unique_tempfile_in(parent, &format!("stripped_{file_name}."), "")?;

    let objcopy = format!("{}objcopy", cross_compile);

    let strip_output = Command::new(&objcopy)
        .arg("--strip-unneeded")
        .arg(binary_path)
        .arg(stripped.path())
        .output()?;

    if !strip_output.status.success() {
        print_output_and_bail(&objcopy, &strip_output)?;
    }

    Ok(stripped)
}

fn sign_ta(config: &TaBuildConfig, stripped_path: &Path) -> Result<BuildArtifact> {
    println!("Signing TA with signing key {:?}...", config.signing_key);

    let uuid = elf_uuid::read_ta_uuid(stripped_path)?;
    let uuid_str = uuid.to_string();

    // Validate signing key exists
    if !config.signing_key.exists() {
        bail!("Signing key not found at {:?}", config.signing_key);
    }

    // Sign script path
    let sign_script = common::join_and_check(
        &config.ta_dev_kit_dir,
        &["scripts", "sign_encrypt.py"],
        "Sign script",
    )?;

    let parent = stripped_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Stripped path has no parent: {:?}", stripped_path))?;
    let output_path = parent.join(format!("{uuid_str}.ta"));
    let tmp = unique_tempfile_in(parent, &format!("{uuid_str}.ta."), ".partial")?;

    let sign_output = Command::new("python3")
        .arg(&sign_script)
        .arg("--uuid")
        .arg(&uuid_str)
        .arg("--key")
        .arg(&config.signing_key)
        .arg("--in")
        .arg(stripped_path)
        .arg("--out")
        .arg(tmp.path())
        .output()?;

    if !sign_output.status.success() {
        print_output_and_bail("sign_encrypt.py", &sign_output)?;
    }

    publish_tempfile(tmp, &output_path)?;
    // NamedTempFile is 0600; sign_encrypt.py truncates without resetting mode.
    set_ta_file_permissions(&output_path)?;

    println!("SIGN => {}", uuid_str);
    let absolute_output_path = output_path
        .canonicalize()
        .unwrap_or_else(|_| output_path.clone());
    println!("TA signed and saved to: {:?}", absolute_output_path);

    Ok(BuildArtifact {
        uuid,
        path: output_path,
    })
}

/// Check if the required cross-compile toolchain is available
fn check_toolchain_exists(cross_compile_prefix: &str) -> Result<()> {
    let gcc_command = format!("{}gcc", cross_compile_prefix);
    let objcopy_command = format!("{}objcopy", cross_compile_prefix);

    // Check if gcc exists
    let gcc_check = Command::new("which").arg(&gcc_command).output();

    // Check if objcopy exists
    let objcopy_check = Command::new("which").arg(&objcopy_command).output();

    let gcc_exists = gcc_check.is_ok_and(|output| output.status.success());
    let objcopy_exists = objcopy_check.is_ok_and(|output| output.status.success());

    if !gcc_exists || !objcopy_exists {
        let missing_tools: Vec<&str> = [
            if !gcc_exists {
                Some(gcc_command.as_str())
            } else {
                None
            },
            if !objcopy_exists {
                Some(objcopy_command.as_str())
            } else {
                None
            },
        ]
        .iter()
        .filter_map(|&x| x)
        .collect();

        eprintln!("Error: Required cross-compile toolchain not found!");
        eprintln!("Missing tools: {}", missing_tools.join(", "));
        eprintln!();
        eprintln!("Please install the required toolchain:");
        eprintln!();
        eprintln!("# For aarch64 host (ARM64 machine):");
        eprintln!("apt update && apt -y install gcc gcc-arm-linux-gnueabihf");
        eprintln!();
        eprintln!("# For x86_64 host (Intel/AMD machine):");
        eprintln!("apt update && apt -y install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf");
        eprintln!();
        eprintln!("Or manually install the cross-compilation tools for your target architecture.");

        bail!("Cross-compile toolchain not available");
    }

    Ok(())
}

// Helper function to setup base command with common environment variables
fn setup_build_command(
    config: &TaBuildConfig,
    command: &str,
) -> Result<(Command, Option<TempDir>)> {
    // Determine target and cross-compile based on arch and std mode
    let build_mode = if config.std {
        BuildMode::TaStd
    } else {
        BuildMode::TaNoStd
    };
    let (target, cross_compile) = resolve_target_and_cross_compile(config.arch, build_mode)?;

    // Setup custom targets if using std - keep TempDir alive
    let temp_dir = if config.std {
        Some(setup_custom_targets()?)
    } else {
        None
    };

    // Always use cargo; std builds use the patched standard library and JSON targets.
    let mut cmd = cargo_command_in(&config.path);
    cmd.arg(command);
    cmd.arg("--manifest-path").arg(manifest_path(&config.path));
    if config.std {
        cmd.arg("-Z").arg("build-std=std,panic_abort");
        cmd.arg("-Z").arg("json-target-spec");
    }
    cmd.arg("--target").arg(&target);

    // Add --no-default-features if specified
    if config.no_default_features {
        cmd.arg("--no-default-features");
    }

    // Build features list
    let mut features = Vec::new();
    if config.std {
        features.push("std".to_string());
    }
    if let Some(ref custom_features) = config.features {
        // Split custom features by comma and add them
        for feature in custom_features.split(',') {
            let feature = feature.trim();
            if !feature.is_empty() {
                features.push(feature.to_string());
            }
        }
    }

    // Add features if any are specified
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    // Set RUSTFLAGS - preserve existing ones and add panic=abort
    let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    if !rustflags.is_empty() {
        rustflags.push(' ');
    }
    rustflags.push_str("-C panic=abort");
    if config.std && !rustflags.contains("unstable-options") {
        rustflags.push_str(" -Z unstable-options");
    }
    cmd.env("RUSTFLAGS", &rustflags);

    // Apply custom environment variables
    for (key, value) in &config.env {
        cmd.env(key, value);
    }

    // Set TA_DEV_KIT_DIR environment variable (use absolute path)
    let absolute_ta_dev_kit_dir = config
        .ta_dev_kit_dir
        .canonicalize()
        .unwrap_or_else(|_| config.ta_dev_kit_dir.clone());
    cmd.env("TA_DEV_KIT_DIR", &absolute_ta_dev_kit_dir);
    // Set CROSS_COMPILE so dependencies with `cc` build scripts pick the
    // cross compiler. The custom OP-TEE targets are absent from `cc`'s
    // builtin table, so without it C code is silently compiled for the host.
    cmd.env("CROSS_COMPILE", cross_compile);

    // Set RUST_TARGET_PATH for custom targets when using std
    if let Some(ref temp_dir_ref) = temp_dir {
        cmd.env("RUST_TARGET_PATH", temp_dir_ref.path());
    }

    // Set __CARGO_TESTS_ONLY_SRC_ROOT for std builds (required by cargo -Z build-std)
    if config.std {
        let rust_src = env::var("__CARGO_TESTS_ONLY_SRC_ROOT")
            .map(PathBuf::from)
            .map_err(|_| {
                anyhow::anyhow!(
                    "__CARGO_TESTS_ONLY_SRC_ROOT is not set.\n\
                For std TA builds, set it to the Rust library source directory, e.g.:\n\
                  export __CARGO_TESTS_ONLY_SRC_ROOT=/path/to/rust/library"
                )
            })?;
        if !rust_src.exists() {
            anyhow::bail!(
                "__CARGO_TESTS_ONLY_SRC_ROOT points to a non-existent path: {:?}",
                rust_src
            );
        }
        cmd.env("__CARGO_TESTS_ONLY_SRC_ROOT", &rust_src);
    }

    Ok((cmd, temp_dir))
}

// Helper function to setup custom target JSONs for std builds
// Returns TempDir to keep it alive during the build
fn setup_custom_targets() -> Result<TempDir> {
    let temp_dir = TempDir::new()?;

    // Write the embedded target JSON files
    let aarch64_path = temp_dir.path().join("aarch64-unknown-optee.json");
    let arm_path = temp_dir.path().join("arm-unknown-optee.json");

    fs::write(aarch64_path, AARCH64_TARGET_JSON)?;
    fs::write(arm_path, ARM_TARGET_JSON)?;

    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Arch;

    /// Guarantee: every TA cargo invocation carries `CROSS_COMPILE`, so
    /// dependencies with `cc` build scripts compile C code for the target.
    /// The custom OP-TEE triples are absent from `cc`'s builtin table;
    /// without this, C code is silently built for the host and linking fails.
    #[test]
    fn ta_commands_carry_cross_compile() -> anyhow::Result<()> {
        let config = TaBuildConfig {
            arch: Arch::Aarch64,
            debug: false,
            path: PathBuf::from("/tmp/project"),
            env: Vec::new(),
            no_default_features: false,
            features: None,
            std: false,
            ta_dev_kit_dir: PathBuf::from("/tmp/ta-dev-kit"),
            signing_key: PathBuf::from("/tmp/key.pem"),
        };
        let (build, _temp) = setup_build_command(&config, "build")?;
        assert_eq!(build.get_current_dir(), Some(Path::new("/tmp/project")));
        let args: Vec<_> = build
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            args.windows(2)
                .any(|w| w[0] == "--manifest-path" && w[1] == "/tmp/project/Cargo.toml"),
            "expected --manifest-path /tmp/project/Cargo.toml, got {args:?}"
        );
        let value = build
            .get_envs()
            .find(|(k, _)| k.to_str() == Some("CROSS_COMPILE"))
            .and_then(|(_, v)| v)
            .map(|v| v.to_string_lossy().into_owned())
            .ok_or_else(|| anyhow::anyhow!("CROSS_COMPILE not set on TA cargo invocation"))?;
        assert_eq!(value, "aarch64-linux-gnu-");
        Ok(())
    }
}
