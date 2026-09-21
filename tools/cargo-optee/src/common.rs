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

use anyhow::{Context, Result, bail};
use cargo_metadata::Message;
use clap::ValueEnum;
use std::env;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use uuid::Uuid;

/// Canonical project directory. Does not change the process cwd.
pub fn absolute_dir(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("project directory {}", path.display()))
}

pub fn manifest_path(project_dir: &Path) -> PathBuf {
    project_dir.join("Cargo.toml")
}

/// Resolve `dir` against the process cwd. Relative install paths must not be
/// interpreted against the crate directory.
pub fn absolute_from_cwd(dir: &Path) -> Result<PathBuf> {
    if dir.is_absolute() {
        Ok(dir.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(dir))
    }
}

/// Cargo invocation rooted at `project_dir` without changing the process cwd.
pub fn cargo_command_in(project_dir: &Path) -> Command {
    let mut cmd = cargo_command();
    cmd.current_dir(project_dir);
    cmd
}

/// Target architecture for building
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Arch {
    /// ARM 64-bit architecture
    Aarch64,
    /// ARM 32-bit architecture
    Arm,
}

impl std::str::FromStr for Arch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aarch64" | "arm64" => Ok(Arch::Aarch64),
            "arm" | "arm32" => Ok(Arch::Arm),
            _ => Err(format!("Invalid architecture: {}", s)),
        }
    }
}

/// Build mode for OP-TEE components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Client Application (CA) - runs in Normal World Linux environment
    /// Can use standard library, uses standard Linux targets
    Ca,
    /// Trusted Application with std support
    /// Uses OP-TEE custom targets (e.g., aarch64-unknown-optee)
    TaStd,
    /// Trusted Application without std support
    /// Uses standard Linux targets but runs in TEE environment
    TaNoStd,
}

/// Target configurations for different architectures and build modes
/// Format: (Architecture, BuildMode, target, cross_compile_prefix)
const TARGET_CONFIGS: [(Arch, BuildMode, &str, &str); 6] = [
    // ARM 32-bit configurations
    (
        Arch::Arm,
        BuildMode::Ca,
        "arm-unknown-linux-gnueabihf",
        "arm-linux-gnueabihf-",
    ),
    (
        Arch::Arm,
        BuildMode::TaNoStd,
        "arm-unknown-linux-gnueabihf",
        "arm-linux-gnueabihf-",
    ),
    (
        Arch::Arm,
        BuildMode::TaStd,
        "arm-unknown-optee",
        "arm-linux-gnueabihf-",
    ),
    // AArch64 configurations
    (
        Arch::Aarch64,
        BuildMode::Ca,
        "aarch64-unknown-linux-gnu",
        "aarch64-linux-gnu-",
    ),
    (
        Arch::Aarch64,
        BuildMode::TaNoStd,
        "aarch64-unknown-linux-gnu",
        "aarch64-linux-gnu-",
    ),
    (
        Arch::Aarch64,
        BuildMode::TaStd,
        "aarch64-unknown-optee",
        "aarch64-linux-gnu-",
    ),
];

/// Unified function to derive target and cross-compile prefix from architecture and build mode
pub fn get_target_and_cross_compile(arch: Arch, mode: BuildMode) -> Result<(String, String)> {
    for &(config_arch, config_mode, target, cross_compile_prefix) in &TARGET_CONFIGS {
        if config_arch == arch && config_mode == mode {
            return Ok((target.to_string(), cross_compile_prefix.to_string()));
        }
    }

    bail!(
        "No target configuration found for arch: {:?}, mode: {:?}",
        arch,
        mode
    )
}

/// Like [`get_target_and_cross_compile`], but honour `CROSS_COMPILE` from the
/// process environment when OP-TEE rust.mk / Make supplied a toolchain prefix.
pub fn resolve_target_and_cross_compile(arch: Arch, mode: BuildMode) -> Result<(String, String)> {
    resolve_target_and_cross_compile_with(
        arch,
        mode,
        env::var("CROSS_COMPILE").ok().filter(|s| !s.is_empty()),
    )
}

fn resolve_target_and_cross_compile_with(
    arch: Arch,
    mode: BuildMode,
    cross_compile_override: Option<String>,
) -> Result<(String, String)> {
    let (target, default_cc) = get_target_and_cross_compile(arch, mode)?;
    Ok((target, cross_compile_override.unwrap_or(default_cc)))
}

/// Helper function to print command output and return error
pub fn print_output_and_bail(cmd_name: &str, output: &Output) -> Result<()> {
    eprintln!(
        "{} stdout: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "{} stderr: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stderr)
    );
    bail!(
        "{} failed with exit code: {:?}",
        cmd_name,
        output.status.code()
    )
}

/// Print cargo command for debugging
pub fn print_cargo_command(cmd: &Command, description: &str) {
    println!("{}...", description);

    // Extract program and args
    let program = cmd.get_program();
    let args: Vec<_> = cmd.get_args().collect();

    // Extract all environment variables
    let envs: Vec<String> = cmd
        .get_envs()
        .filter_map(|(k, v)| match (k.to_str(), v.and_then(|v| v.to_str())) {
            (Some(key), Some(value)) => Some(format!("{}={}", key, value)),
            _ => None,
        })
        .collect();

    // Print environment variables
    if !envs.is_empty() {
        println!("  Environment: {}", envs.join(" "));
    }

    if let Some(dir) = cmd.get_current_dir() {
        println!("  Working directory: {}", dir.display());
    }

    // Print command
    println!(
        "  Command: {} {}",
        program.to_string_lossy(),
        args.into_iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
}

/// Invoke Cargo, honoring the `CARGO` environment variable.
pub fn cargo_command() -> Command {
    const ENV_CARGO: &str = "CARGO";
    Command::new(std::env::var_os(ENV_CARGO).unwrap_or_else(|| "cargo".into()))
}

/// Artifact produced by a TA or plugin build, named from the UUID parsed out of
/// the unsigned ELF rather than from any external configuration.
#[derive(Debug, Clone)]
pub struct BuildArtifact {
    pub uuid: Uuid,
    pub path: PathBuf,
}

/// Kind of Cargo compiler artifact to collect from `--message-format=json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CargoArtifactKind {
    Bin,
    Cdylib,
}

/// Collect the current package's compiler artifacts from `cargo --message-format=json`.
pub fn collect_cargo_artifacts(
    stdout: &[u8],
    manifest_path: &std::path::Path,
    kind: CargoArtifactKind,
) -> Result<Vec<PathBuf>> {
    let expected =
        std::fs::canonicalize(manifest_path).unwrap_or_else(|_| manifest_path.to_path_buf());
    let mut artifacts = Vec::new();

    for message in Message::parse_stream(Cursor::new(stdout)) {
        let message = match message {
            Ok(message) => message,
            Err(_) => continue,
        };
        match message {
            Message::CompilerMessage(msg) => {
                if let Some(rendered) = &msg.message.rendered {
                    eprint!("{rendered}");
                }
            }
            Message::CompilerArtifact(artifact) => {
                let artifact_manifest = artifact.manifest_path.into_std_path_buf();
                let artifact_manifest =
                    std::fs::canonicalize(&artifact_manifest).unwrap_or(artifact_manifest);
                if artifact_manifest != expected {
                    continue;
                }
                if artifact.profile.test {
                    continue;
                }
                if artifact.target.kind.iter().any(|k| k == "custom-build") {
                    continue;
                }
                match kind {
                    CargoArtifactKind::Bin => {
                        if let Some(path) = artifact.executable {
                            artifacts.push(path.into_std_path_buf());
                        }
                    }
                    CargoArtifactKind::Cdylib => {
                        if !artifact.target.crate_types.iter().any(|t| t == "cdylib") {
                            continue;
                        }
                        for path in artifact.filenames {
                            if path.as_str().ends_with(".so") {
                                artifacts.push(path.into_std_path_buf());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(artifacts)
}

pub fn unique_cargo_artifact(
    stdout: &[u8],
    manifest_path: &std::path::Path,
    kind: CargoArtifactKind,
) -> Result<PathBuf> {
    let mut artifacts = collect_cargo_artifacts(stdout, manifest_path, kind)?;
    artifacts.sort();
    artifacts.dedup();
    match artifacts.len() {
        1 => Ok(artifacts.remove(0)),
        0 => bail!(
            "Cargo produced no {:?} artifact for {}",
            kind,
            manifest_path.display()
        ),
        _ => bail!(
            "Cargo produced multiple {:?} artifacts for {}: {:?}. Use a single bin/cdylib target.",
            kind,
            manifest_path.display(),
            artifacts
        ),
    }
}

/// Create a unique tempfile in `parent` so a later rename onto a sibling path
/// stays on the same filesystem.
pub fn unique_tempfile_in(
    parent: &Path,
    prefix: &str,
    suffix: &str,
) -> Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile_in(parent)
        .with_context(|| format!("create unique temp in {}", parent.display()))
}

/// Move a unique tempfile onto `dest`, replacing it on Unix.
pub fn publish_tempfile(tmp: tempfile::NamedTempFile, dest: &Path) -> Result<()> {
    let (_file, path) = tmp.keep().map_err(|e| e.error)?;
    fs::rename(&path, dest)
        .with_context(|| format!("failed to publish {} to {}", path.display(), dest.display()))
}

/// Mode for signed `.ta` files. `NamedTempFile` is 0600; `sign_encrypt.py`
/// truncates `--out` without resetting it. tee-supplicant may not be the
/// build user, so keep the artifact world-readable.
pub const TA_FILE_MODE: u32 = 0o644;

pub fn set_ta_file_permissions(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(TA_FILE_MODE))
        .with_context(|| format!("set permissions on {}", path.display()))
}

/// Deny lints used by `cargo-optee clippy` and the former in-build clippy step.
pub fn apply_clippy_deny_lints(cmd: &mut Command) {
    cmd.arg("--");
    cmd.arg("-D").arg("warnings");
    cmd.arg("-D").arg("clippy::unwrap_used");
    cmd.arg("-D").arg("clippy::expect_used");
    cmd.arg("-D").arg("clippy::panic");
}

/// Join path segments and check if the resulting path exists
pub fn join_and_check<P: AsRef<Path>>(
    base: &Path,
    segments: &[P],
    error_context: &str,
) -> Result<PathBuf> {
    let mut path = base.to_path_buf();
    for segment in segments {
        path = path.join(segment);
    }

    if !path.exists() {
        bail!("{} does not exist: {:?}", error_context, path);
    }

    Ok(path)
}

/// Clean build artifacts for any OP-TEE component (TA, CA, Plugin)
pub fn clean_project(project_path: &std::path::Path) -> Result<()> {
    let project_path = absolute_dir(project_path)?;
    println!("Cleaning build artifacts in: {:?}", project_path);

    let output = cargo_command_in(&project_path)
        .arg("clean")
        .arg("--manifest-path")
        .arg(manifest_path(&project_path))
        .output()?;

    if !output.status.success() {
        print_output_and_bail("cargo clean", &output)?;
    }

    // Also clean the intermediate cargo-optee directory if it exists
    let intermediate_dir = project_path.join("target").join("cargo-optee");
    if intermediate_dir.exists() {
        fs::remove_dir_all(&intermediate_dir)?;
        println!("Removed intermediate directory: {:?}", intermediate_dir);
    }

    println!("Build artifacts cleaned successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guarantee: cargo is rooted at the crate via child cwd; the process
    /// cwd is unchanged so relative `--target-dir` follows make/emulate.
    #[test]
    fn cargo_command_in_sets_child_cwd_only() {
        let dir = PathBuf::from("/tmp/project");
        let cmd = cargo_command_in(&dir);
        assert_eq!(cmd.get_current_dir(), Some(dir.as_path()));
    }

    /// Guarantee: a non-empty `CROSS_COMPILE` replaces the arch default
    /// prefix, so `cc` build scripts use the OP-TEE toolchain.
    #[test]
    fn cross_compile_override_replaces_default() {
        let (_, cc) = resolve_target_and_cross_compile_with(
            Arch::Aarch64,
            BuildMode::TaNoStd,
            Some("custom-none-".into()),
        )
        .unwrap();
        assert_eq!(cc, "custom-none-");
    }

    /// Guarantee: two temps with the same prefix/suffix in one directory
    /// get distinct paths, so parallel strip/sign cannot clobber each other.
    #[test]
    fn unique_tempfiles_do_not_share_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let a = unique_tempfile_in(dir.path(), "ta.", ".partial").unwrap();
        let b = unique_tempfile_in(dir.path(), "ta.", ".partial").unwrap();
        assert_ne!(a.path(), b.path());
    }

    /// Guarantee: publishing onto an existing path replaces its contents,
    /// so a rebuild cannot leave make/emulate installing a stale `.ta`.
    #[test]
    fn publish_tempfile_replaces_destination() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.ta");
        std::fs::write(&dest, b"old").unwrap();
        let tmp = unique_tempfile_in(dir.path(), "ta.", ".partial").unwrap();
        std::fs::write(tmp.path(), b"new").unwrap();
        publish_tempfile(tmp, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"new");
    }

    /// Guarantee: `unique_tempfile_in` creates a 0600 file. `sign_encrypt.py`
    /// truncates `--out` without resetting mode, so this is the `.ta` mode
    /// unless `set_ta_file_permissions` runs after publish.
    #[test]
    fn unique_tempfile_is_owner_only() {
        let dir = tempfile::tempdir().unwrap();
        let tmp = unique_tempfile_in(dir.path(), "ta.", ".partial").unwrap();
        let mode = fs::metadata(tmp.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    /// Guarantee: after `set_ta_file_permissions`, the published `.ta` and
    /// the install copy are 0644, so tee-supplicant can read them.
    #[test]
    fn published_ta_is_world_readable() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("out.ta");
        let tmp = unique_tempfile_in(dir.path(), "ta.", ".partial").unwrap();
        std::fs::write(tmp.path(), b"ta").unwrap();
        publish_tempfile(tmp, &dest).unwrap();
        set_ta_file_permissions(&dest).unwrap();
        let mode = fs::metadata(&dest).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, TA_FILE_MODE);

        let installed = dir.path().join("installed.ta");
        fs::copy(&dest, &installed).unwrap();
        set_ta_file_permissions(&installed).unwrap();
        let installed_mode = fs::metadata(&installed).unwrap().permissions().mode() & 0o777;
        assert_eq!(installed_mode, TA_FILE_MODE);
    }

    /// Guarantee: an absolute `--target-dir` is returned unchanged, not
    /// joined onto the process cwd.
    #[test]
    fn absolute_from_cwd_keeps_absolute_paths() {
        let abs = PathBuf::from("/tmp/dist");
        assert_eq!(absolute_from_cwd(&abs).unwrap(), abs);
    }

    /// Guarantee: unset `CROSS_COMPILE` still yields the arch default
    /// prefix, so `cc` build scripts do not compile C for the host.
    #[test]
    fn cross_compile_default_when_unset() {
        let (_, cc) =
            resolve_target_and_cross_compile_with(Arch::Aarch64, BuildMode::TaNoStd, None).unwrap();
        assert_eq!(cc, "aarch64-linux-gnu-");
    }
}
