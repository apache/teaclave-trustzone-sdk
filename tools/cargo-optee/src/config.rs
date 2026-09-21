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

use anyhow::{Result, bail};
use cargo_metadata::MetadataCommand;
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};

use crate::common::Arch;

/// Component type for OP-TEE builds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    /// Trusted Application (TA)
    Ta,
    /// Client Application (CA)
    Ca,
    /// Plugin (Shared Library)
    Plugin,
}

impl ComponentType {
    /// Convert to string for metadata lookup in Cargo.toml
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::Ta => "ta",
            ComponentType::Ca => "ca",
            ComponentType::Plugin => "plugin",
        }
    }
}

/// Path type for validation
enum PathType {
    /// Expects a directory
    Directory,
    /// Expects a file
    File,
}

#[derive(Clone)]
pub struct TaBuildConfig {
    pub arch: Arch,    // Architecture
    pub debug: bool,   // Debug mode (default false = release)
    pub path: PathBuf, // Path to TA directory
    // Customized variables
    pub env: Vec<(String, String)>, // Custom environment variables for cargo build
    pub no_default_features: bool,  // Disable default features
    pub features: Option<String>,   // Additional features to enable
    // ta specific variables
    pub std: bool,               // Enable std feature
    pub ta_dev_kit_dir: PathBuf, // Path to TA dev kit
    pub signing_key: PathBuf,    // Path to signing key
}

/// CLI overrides shared by TA and CA/plugin resolution.
pub struct BuildOverrides {
    pub arch: Option<Arch>,
    pub debug: Option<bool>,
    pub env: Vec<(String, String)>,
    pub no_default_features: bool,
    pub features: Option<String>,
}

impl TaBuildConfig {
    pub fn resolve(
        project_path: &Path,
        overrides: BuildOverrides,
        cmd_std: Option<bool>,
        cmd_ta_dev_kit_dir: Option<PathBuf>,
        cmd_signing_key: Option<PathBuf>,
        need_signing_key: bool,
    ) -> Result<Self> {
        // Get base configuration from metadata
        let metadata_config =
            MetadataConfig::resolve(project_path, ComponentType::Ta, overrides.arch)?;

        // Determine final arch: CLI > metadata > default
        let arch = overrides
            .arch
            .or_else(|| metadata_config.as_ref().map(|c| c.arch))
            .unwrap_or(Arch::Aarch64);

        // Handle priority: CLI > metadata > default
        let debug = overrides
            .debug
            .or_else(|| metadata_config.as_ref().map(|c| c.debug))
            .unwrap_or(false);

        let std = cmd_std
            .or_else(|| metadata_config.as_ref().map(|c| c.std))
            .unwrap_or(false);

        // Handle ta_dev_kit_dir: CLI > metadata > TA_DEV_KIT_DIR > error (required)
        let ta_dev_kit_dir_config = resolve_config_path(
            cmd_ta_dev_kit_dir,
            metadata_config
                .as_ref()
                .and_then(|c| c.ta_dev_kit_dir.clone()),
            "TA_DEV_KIT_DIR",
        )
        .ok_or_else(ta_dev_kit_dir_error)?;

        // Resolve ta_dev_kit_dir path (relative to absolute)
        let ta_dev_kit_dir = resolve_path_relative_to_project(
            &ta_dev_kit_dir_config,
            project_path,
            PathType::Directory,
            "TA development kit directory",
        )?;

        // Handle signing_key: CLI > metadata > TA_SIGN_KEY > default_ta.pem.
        // Clippy does not sign, so skip the key file when linting.
        let signing_key = if need_signing_key {
            let signing_key_config = cmd_signing_key
                .or_else(|| metadata_config.as_ref().and_then(|c| c.signing_key.clone()))
                .or_else(|| env::var_os("TA_SIGN_KEY").map(PathBuf::from))
                .unwrap_or_else(|| ta_dev_kit_dir_config.join("keys").join("default_ta.pem"));
            resolve_path_relative_to_project(
                &signing_key_config,
                project_path,
                PathType::File,
                "Signing key file",
            )?
        } else {
            PathBuf::new()
        };

        // Merge environment variables: metadata env + CLI env (CLI overrides metadata)
        let mut env = metadata_config
            .as_ref()
            .map(|c| c.env.clone())
            .unwrap_or_default();
        env.extend(overrides.env);

        Ok(TaBuildConfig {
            arch,
            debug,
            std,
            ta_dev_kit_dir,
            signing_key,
            path: project_path.to_path_buf(),
            env,
            no_default_features: overrides.no_default_features,
            features: overrides.features,
        })
    }

    /// Print the final TA configuration parameters being used
    pub fn print_config(&self) {
        self.print_config_with("Building");
    }

    pub fn print_lint_config(&self) {
        self.print_config_with("Linting");
    }

    fn print_config_with(&self, action: &str) {
        println!("{} TA with:", action);
        println!("  Arch: {:?}", self.arch);
        println!("  Debug: {}", self.debug);
        println!("  Std: {}", self.std);
        println!("  TA dev kit dir: {:?}", self.ta_dev_kit_dir);
        if !self.signing_key.as_os_str().is_empty() {
            println!("  Signing key: {:?}", self.signing_key);
        }
        if !self.env.is_empty() {
            println!("  Environment variables: {} set", self.env.len());
        }
    }
}

#[derive(Clone)]
pub struct CaBuildConfig {
    pub arch: Arch,    // Architecture
    pub debug: bool,   // Debug mode (default false = release)
    pub path: PathBuf, // Path to CA directory
    // Customized variables
    pub env: Vec<(String, String)>, // Custom environment variables for cargo build
    pub no_default_features: bool,  // Disable default features
    pub features: Option<String>,   // Additional features to enable
    // ca specific variables
    pub optee_client_export: PathBuf, // Path to OP-TEE client export
    pub plugin: bool,                 // Build as plugin (shared library)
}

impl CaBuildConfig {
    pub fn resolve(
        project_path: &Path,
        overrides: BuildOverrides,
        cmd_optee_client_export: Option<PathBuf>,
        plugin: bool,
    ) -> Result<Self> {
        let component_type = if plugin {
            ComponentType::Plugin
        } else {
            ComponentType::Ca
        };

        // Get base configuration from metadata
        let metadata_config =
            MetadataConfig::resolve(project_path, component_type, overrides.arch)?;

        // Determine final arch: CLI > metadata > default
        let arch = overrides
            .arch
            .or_else(|| metadata_config.as_ref().map(|c| c.arch))
            .unwrap_or(Arch::Aarch64);

        // Handle priority: CLI > metadata > default
        let debug = overrides
            .debug
            .or_else(|| metadata_config.as_ref().map(|c| c.debug))
            .unwrap_or(false);

        // Handle client export: CLI > metadata > OPTEE_CLIENT_EXPORT > error (required)
        let optee_client_export_config = resolve_config_path(
            cmd_optee_client_export,
            metadata_config
                .as_ref()
                .and_then(|c| c.optee_client_export.clone()),
            "OPTEE_CLIENT_EXPORT",
        )
        .ok_or_else(optee_client_export_error)?;

        // Resolve optee_client_export path (relative to absolute)
        let optee_client_export = resolve_path_relative_to_project(
            &optee_client_export_config,
            project_path,
            PathType::Directory,
            "OP-TEE client export directory",
        )?;

        // Merge environment variables: metadata env + CLI env (CLI overrides metadata)
        let mut env = metadata_config
            .as_ref()
            .map(|c| c.env.clone())
            .unwrap_or_default();
        env.extend(overrides.env);

        Ok(CaBuildConfig {
            arch,
            debug,
            path: project_path.to_path_buf(),
            env,
            no_default_features: overrides.no_default_features,
            features: overrides.features,
            optee_client_export,
            plugin,
        })
    }

    /// Print the final CA/Plugin configuration parameters being used
    pub fn print_config(&self) {
        self.print_config_with("Building");
    }

    pub fn print_lint_config(&self) {
        self.print_config_with("Linting");
    }

    fn print_config_with(&self, action: &str) {
        let component_name = if self.plugin { "Plugin" } else { "CA" };
        println!("{} {} with:", action, component_name);
        println!("  Arch: {:?}", self.arch);
        println!("  Debug: {}", self.debug);
        println!("  OP-TEE client export: {:?}", self.optee_client_export);
        if !self.env.is_empty() {
            println!("  Environment variables: {} set", self.env.len());
        }
    }
}

/// Build configuration parsed from Cargo.toml metadata only
/// This struct is used internally for metadata parsing and does not handle priority resolution
#[derive(Debug, Clone)]
struct MetadataConfig {
    pub arch: Arch,
    pub debug: bool,
    pub std: bool,
    pub ta_dev_kit_dir: Option<PathBuf>,
    pub optee_client_export: Option<PathBuf>,
    pub signing_key: Option<PathBuf>,
    /// additional environment key-value pairs, that should be passed to underlying
    /// build commands
    pub env: Vec<(String, String)>,
}

impl MetadataConfig {
    /// Extract build configuration from metadata only.
    /// Returns `Ok(None)` when the package has no `[package.metadata.optee.<kind>]`
    /// table. Parse errors, including removed keys, are returned to the CLI.
    pub fn resolve(
        project_path: &Path,
        component_type: ComponentType,
        cmd_arch: Option<Arch>,
    ) -> Result<Option<Self>> {
        let app_metadata = discover_app_metadata(project_path)?;
        extract_build_config(&app_metadata, component_type, cmd_arch)
    }
}

/// Discover application metadata from the current project
fn discover_app_metadata(project_path: &Path) -> Result<Value> {
    let cargo_toml_path = project_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        bail!(
            "Cargo.toml not found in project directory: {:?}",
            project_path
        );
    }

    // Get metadata for the current project
    let metadata = MetadataCommand::new()
        .manifest_path(&cargo_toml_path)
        .no_deps()
        .exec()?;

    // Find the current project package
    // First try to get root package (for non-workspace projects)
    let current_package = if let Some(root_pkg) = metadata.root_package() {
        root_pkg
    } else {
        // For workspace projects, find the package that corresponds to this manifest
        let cargo_toml_path_str = cargo_toml_path.to_string_lossy();
        metadata
            .packages
            .iter()
            .find(|pkg| {
                pkg.manifest_path
                    .to_string()
                    .contains(&*cargo_toml_path_str)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not find package for manifest: {}",
                    cargo_toml_path_str
                )
            })?
    };

    Ok(current_package.metadata.clone())
}

/// Extract build configuration from application package metadata.
fn extract_build_config(
    metadata: &Value,
    component_type: ComponentType,
    cmd_arch: Option<Arch>,
) -> Result<Option<MetadataConfig>> {
    let Some(optee_metadata) = metadata.get("optee") else {
        return Ok(None);
    };
    let Some(component_metadata) = optee_metadata.get(component_type.as_str()) else {
        return Ok(None);
    };

    if component_metadata.get("uuid-path").is_some() {
        bail!(
            "package.metadata.optee.{}.uuid-path is not supported; UUID is parsed from the unsigned ELF",
            component_type.as_str()
        );
    }

    let arch = match cmd_arch {
        Some(arch) => arch,
        None => match component_metadata.get("arch") {
            None => Arch::Aarch64,
            Some(v) => {
                let s = v
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("metadata key arch must be a string"))?;
                s.parse()
                    .map_err(|e: String| anyhow::anyhow!("invalid metadata arch {s:?}: {e}"))?
            }
        },
    };

    let debug = metadata_bool(component_metadata, "debug")?;
    let std = metadata_bool(component_metadata, "std")?;

    // Architecture-specific path resolution
    let arch_key = match arch {
        Arch::Aarch64 => "aarch64",
        Arch::Arm => "arm",
    };

    // Parse architecture-specific ta_dev_kit_dir (for TA only)
    let ta_dev_kit_dir = if component_type == ComponentType::Ta {
        component_metadata
            .get("ta-dev-kit-dir")
            .and_then(|v| {
                // Try architecture-specific first
                if let Some(arch_value) = v.get(arch_key) {
                    // Only accept string values, no null support
                    arch_value.as_str()
                } else {
                    // Architecture key missing, try fallback to non-specific
                    if v.is_string() { v.as_str() } else { None }
                }
            })
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    } else {
        None
    };

    // Parse architecture-specific optee_client_export (for CA and Plugin)
    let optee_client_export =
        if component_type == ComponentType::Ca || component_type == ComponentType::Plugin {
            component_metadata
                .get("optee-client-export")
                .and_then(|v| {
                    // Try architecture-specific first
                    if let Some(arch_value) = v.get(arch_key) {
                        // Only accept string values, no null support
                        arch_value.as_str()
                    } else {
                        // Architecture key missing, try fallback to non-specific
                        if v.is_string() { v.as_str() } else { None }
                    }
                })
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        } else {
            None
        };

    // Parse signing key (for TA only)
    let signing_key = if component_type == ComponentType::Ta {
        component_metadata
            .get("signing-key")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    } else {
        None
    };

    // Parse environment variables
    let env: Vec<(String, String)> = component_metadata
        .get("env")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| {
                    if let Some(eq_pos) = s.find('=') {
                        let (key, value) = s.split_at(eq_pos);
                        let value = &value[1..]; // Skip the '=' character
                        Some((key.to_string(), value.to_string()))
                    } else {
                        eprintln!("Warning: Invalid environment variable format in metadata: '{}'. Expected 'KEY=VALUE'", s);
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Some(MetadataConfig {
        arch,
        debug,
        std,
        ta_dev_kit_dir,
        optee_client_export,
        signing_key,
        env,
    }))
}

fn metadata_bool(table: &Value, key: &str) -> Result<bool> {
    match table.get(key) {
        None => Ok(false),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("metadata key {key} must be a boolean")),
    }
}

/// Select a path from explicit configuration, then fall back to an environment variable.
fn resolve_config_path(
    command_line: Option<PathBuf>,
    metadata: Option<PathBuf>,
    environment_variable: &str,
) -> Option<PathBuf> {
    command_line
        .or(metadata)
        .or_else(|| std::env::var_os(environment_variable).map(PathBuf::from))
}

/// Generate error message for missing ta-dev-kit-dir configuration
fn ta_dev_kit_dir_error() -> anyhow::Error {
    anyhow::anyhow!(
        "ta-dev-kit-dir is MANDATORY but not configured.\n\
        Please set it via:\n\
        1. Command line: --ta-dev-kit-dir <path>\n\
        2. Cargo.toml metadata: [package.metadata.optee.ta] section\n\
        3. Environment variable: TA_DEV_KIT_DIR\n\
        \n\
        Example Cargo.toml:\n\
        [package.metadata.optee.ta]\n\
        ta-dev-kit-dir = {{ aarch64 = \"/path/to/optee_os/out/arm-plat-vexpress/export-ta_arm64\" }}\n\
        \n\
        For help with available options, run: cargo-optee build ta --help"
    )
}

/// Generate error message for missing optee-client-export configuration
fn optee_client_export_error() -> anyhow::Error {
    anyhow::anyhow!(
        "optee-client-export is MANDATORY but not configured.\n\
        Please set it via:\n\
        1. Command line: --optee-client-export <path>\n\
        2. Cargo.toml metadata: [package.metadata.optee.ca] or [package.metadata.optee.plugin] section\n\
        3. Environment variable: OPTEE_CLIENT_EXPORT\n\
        \n\
        Example Cargo.toml:\n\
        [package.metadata.optee.ca]\n\
        optee-client-export = {{ aarch64 = \"/path/to/optee_client/export_arm64\" }}\n\
        \n\
        For help with available options, run: cargo-optee build ca --help"
    )
}

/// Resolve a potentially relative path to an absolute path based on the project directory
/// and validate that it exists
fn resolve_path_relative_to_project(
    path: &PathBuf,
    project_path: &std::path::Path,
    path_type: PathType,
    error_context: &str,
) -> anyhow::Result<PathBuf> {
    let resolved_path = if path.is_absolute() {
        path.clone()
    } else {
        project_path.join(path)
    };

    // Validate that the path exists
    if !resolved_path.exists() {
        bail!("{} does not exist: {:?}", error_context, resolved_path);
    }

    // Additional validation: check if it's actually a directory or file as expected
    match path_type {
        PathType::Directory => {
            if !resolved_path.is_dir() {
                bail!("{} is not a directory: {:?}", error_context, resolved_path);
            }
        }
        PathType::File => {
            if !resolved_path.is_file() {
                bail!("{} is not a file: {:?}", error_context, resolved_path);
            }
        }
    }

    Ok(resolved_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ta_meta(ta: Value) -> Value {
        json!({ "optee": { "ta": ta } })
    }

    /// Guarantee: missing `debug` / `std` in metadata are false (release
    /// and no-std).
    #[test]
    fn debug_and_std_default_false() {
        let cfg = extract_build_config(&ta_meta(json!({})), ComponentType::Ta, None)
            .unwrap()
            .unwrap();
        assert!(!cfg.debug);
        assert!(!cfg.std);
    }

    /// Guarantee: metadata `debug = true` is extracted as true.
    #[test]
    fn debug_true_is_honored() {
        let cfg = extract_build_config(&ta_meta(json!({ "debug": true })), ComponentType::Ta, None)
            .unwrap()
            .unwrap();
        assert!(cfg.debug);
        assert!(!cfg.std);
    }

    /// Guarantee: a non-boolean `debug` value is an error, not a silent
    /// `false` that always builds `--release`.
    #[test]
    fn debug_rejects_non_bool() {
        let err = extract_build_config(
            &ta_meta(json!({ "debug": "true" })),
            ComponentType::Ta,
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("debug"));
        assert!(err.to_string().contains("boolean"));
    }

    /// Guarantee: a non-boolean `std` value is an error, not a silent
    /// `false` that keeps `--no-std`.
    #[test]
    fn std_rejects_non_bool() {
        let err = extract_build_config(&ta_meta(json!({ "std": 1 })), ComponentType::Ta, None)
            .unwrap_err();
        assert!(err.to_string().contains("std"));
        assert!(err.to_string().contains("boolean"));
    }
}
