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
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Run a command with inherited stdout/stderr so its output streams as it is
/// produced instead of being buffered in memory. Stdin stays closed, matching
/// `Command::output()` and preventing accidental terminal reads.
pub fn run_streaming_command(cmd: &mut Command, cmd_name: &str) -> Result<()> {
    let status = cmd
        .stdin(Stdio::null())
        .status()
        .with_context(|| format!("failed to run {cmd_name}"))?;
    if !status.success() {
        bail!("{cmd_name} failed with exit code: {:?}", status.code());
    }
    Ok(())
}

/// Check whether a command can be resolved and launched via the same PATH
/// lookup used by later `Command::new` invocations. The exit status is ignored
/// so tools without `--version` support are still reported as available.
pub fn check_command_available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
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

/// Consume `cargo --message-format=json` line by line, printing diagnostics as
/// they arrive and collecting the current package's compiler artifacts.
pub fn collect_cargo_artifacts<R: BufRead>(
    reader: R,
    manifest_path: &std::path::Path,
    kind: CargoArtifactKind,
) -> Result<Vec<PathBuf>> {
    let expected =
        std::fs::canonicalize(manifest_path).unwrap_or_else(|_| manifest_path.to_path_buf());
    let mut artifacts = Vec::new();

    for message in Message::parse_stream(reader) {
        let message = message?;
        match message {
            Message::CompilerMessage(msg) => {
                if let Some(rendered) = &msg.message.rendered {
                    eprint!("{rendered}");
                }
            }
            Message::TextLine(line) => {
                eprintln!("{line}");
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

/// Run a cargo build that emits JSON messages to stdout. Diagnostics are
/// streamed to stderr while the child runs; only the artifact paths needed for
/// the post-build steps are retained.
pub fn run_cargo_build(
    cmd: &mut Command,
    manifest_path: &std::path::Path,
    kind: CargoArtifactKind,
) -> Result<Vec<PathBuf>> {
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to run {:?}", cmd.get_program()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("cargo stdout was not piped"))?;

    // Drain stdout to EOF before waiting so a full pipe cannot deadlock the
    // child. The reader processes one JSON line at a time.
    let artifacts = collect_cargo_artifacts(BufReader::new(stdout), manifest_path, kind);
    let status = child.wait().context("failed to wait for cargo build")?;

    // Prefer the build failure status over a stdout read error: the status is
    // the actionable cause for the user.
    if !status.success() {
        bail!("build failed with exit code: {:?}", status.code());
    }

    artifacts
}

pub fn unique_cargo_artifact(
    mut artifacts: Vec<PathBuf>,
    manifest_path: &std::path::Path,
    kind: CargoArtifactKind,
) -> Result<PathBuf> {
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

    let mut cmd = cargo_command_in(&project_path);
    cmd.arg("clean")
        .arg("--manifest-path")
        .arg(manifest_path(&project_path));
    run_streaming_command(&mut cmd, "cargo clean")?;

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

    fn fake_artifact_line(manifest: &Path, executable: &Path) -> String {
        serde_json::json!({
            "reason": "compiler-artifact",
            "package_id": "demo 0.1.0 (path+file:///demo)",
            "manifest_path": manifest,
            "target": {
                "kind": ["bin"],
                "crate_types": ["bin"],
                "name": "demo",
                "src_path": "/demo/src/main.rs",
                "edition": "2021",
                "doctest": false
            },
            "profile": {
                "opt_level": "0",
                "debuginfo": 2,
                "debug_assertions": true,
                "overflow_checks": true,
                "test": false
            },
            "features": [],
            "filenames": [executable],
            "executable": executable,
            "fresh": false
        })
        .to_string()
    }

    /// Guarantee: the streamed JSON reader still selects the current
    /// package's executable, so streaming does not lose the build artifact.
    #[test]
    fn streamed_cargo_build_collects_current_package_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(&manifest, "").unwrap();
        let executable = dir.path().join("demo");
        let line = fake_artifact_line(&manifest, &executable);

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg("printf '%s\\n' \"$1\"")
            .arg("sh")
            .arg(line);

        let artifacts = run_cargo_build(&mut cmd, &manifest, CargoArtifactKind::Bin).unwrap();
        assert_eq!(artifacts, vec![executable]);
    }

    /// Guarantee: a failing streamed build returns an error carrying the child
    /// exit code, so a compiler failure cannot be mistaken for success.
    #[test]
    fn streamed_cargo_build_reports_child_failure() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        fs::write(&manifest, "").unwrap();

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("exit 7");

        let err = run_cargo_build(&mut cmd, &manifest, CargoArtifactKind::Bin).unwrap_err();
        assert!(
            err.to_string().contains("exit code: Some(7)"),
            "unexpected error: {err}"
        );
    }
}
