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

use anyhow::{bail, Result};
use cargo_metadata::MetadataCommand;
use serde_json::Value;
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
    pub arch: Arch,                 // Architecture
    pub debug: bool,                // Debug mode (default false = release)
    pub path: PathBuf,              // Path to TA directory
    pub uuid_path: Option<PathBuf>, // Path to UUID file
    // Customized variables
    pub env: Vec<(String, String)>, // Custom environment variables for cargo build
    pub no_default_features: bool,  // Disable default features
    pub features: Option<String>,   // Additional features to enable
    // ta specific variables
    pub std: bool,               // Enable std feature
    pub ta_dev_kit_dir: PathBuf, // Path to TA dev kit
    pub signing_key: PathBuf,    // Path to signing key
}

impl TaBuildConfig {
    pub fn resolve(
        project_path: &Path,
        cmd_arch: Option<Arch>,
        cmd_debug: Option<bool>,
        cmd_uuid_path: Option<PathBuf>,
        common_env: Vec<(String, String)>,
        common_no_default_features: bool,
        common_features: Option<String>,
        cmd_std: Option<bool>,
        cmd_ta_dev_kit_dir: Option<PathBuf>,
        cmd_signing_key: Option<PathBuf>,
    ) -> Result<Self> {
        // Get base configuration from metadata
        let metadata_config = MetadataConfig::resolve(project_path, ComponentType::Ta, cmd_arch)?;

        // Determine final arch: CLI > metadata > default
        let arch = cmd_arch
            .or_else(|| metadata_config.as_ref().map(|c| c.arch))
            .unwrap_or(Arch::Aarch64);

        // Handle priority: CLI > metadata > default
        let debug = cmd_debug
            .or_else(|| metadata_config.as_ref().map(|c| c.debug))
            .unwrap_or(false);

        let std = cmd_std
            .or_else(|| metadata_config.as_ref().map(|c| c.std))
            .unwrap_or(false);

        // Handle ta_dev_kit_dir: CLI > metadata > error (required)
        let ta_dev_kit_dir_config = cmd_ta_dev_kit_dir
            .or_else(|| {
                metadata_config
                    .as_ref()
                    .and_then(|c| c.ta_dev_kit_dir.clone())
            })
            .ok_or_else(|| ta_dev_kit_dir_error())?;

        // Resolve ta_dev_kit_dir path (relative to absolute)
        let ta_dev_kit_dir = resolve_path_relative_to_project(
            &ta_dev_kit_dir_config,
            project_path,
            PathType::Directory,
            "TA development kit directory",
        )?;

        // Handle signing_key: CLI > metadata > default (ta_dev_kit_dir/keys/default_ta.pem)
        let signing_key_config = cmd_signing_key
            .or_else(|| metadata_config.as_ref().and_then(|c| c.signing_key.clone()))
            .unwrap_or_else(|| ta_dev_kit_dir_config.join("keys").join("default_ta.pem"));

        // Resolve signing_key path (relative to absolute)
        let signing_key = resolve_path_relative_to_project(
            &signing_key_config,
            project_path,
            PathType::File,
            "Signing key file",
        )?;

        // Handle uuid_path: CLI > metadata > default (../uuid.txt)
        let uuid_path = resolve_uuid_path(
            cmd_uuid_path,
            metadata_config.as_ref().and_then(|c| c.uuid_path.clone()),
            project_path,
            PathBuf::from("../uuid.txt"),
        )?;

        // Merge environment variables: metadata env + CLI env (CLI overrides metadata)
        let mut env = metadata_config
            .as_ref()
            .map(|c| c.env.clone())
            .unwrap_or_default();
        env.extend(common_env);

        Ok(TaBuildConfig {
            arch,
            debug,
            std,
            ta_dev_kit_dir,
            signing_key,
            path: project_path.to_path_buf(),
            uuid_path: Some(uuid_path),
            env,
            no_default_features: common_no_default_features,
            features: common_features,
        })
    }

    /// Print the final TA configuration parameters being used
    pub fn print_config(&self) {
        println!("Building TA with:");
        println!("  Arch: {:?}", self.arch);
        println!("  Debug: {}", self.debug);
        println!("  Std: {}", self.std);
        println!("  TA dev kit dir: {:?}", self.ta_dev_kit_dir);
        println!("  Signing key: {:?}", self.signing_key);
        if let Some(ref uuid_path) = self.uuid_path {
            let absolute_uuid_path = uuid_path
                .canonicalize()
                .unwrap_or_else(|_| uuid_path.clone());
            println!("  UUID path: {:?}", absolute_uuid_path);
        }
        if !self.env.is_empty() {
            println!("  Environment variables: {} set", self.env.len());
        }
    }
}

#[derive(Clone)]
pub struct CaBuildConfig {
    pub arch: Arch,                 // Architecture
    pub debug: bool,                // Debug mode (default false = release)
    pub path: PathBuf,              // Path to CA directory
    pub uuid_path: Option<PathBuf>, // Path to UUID file (for plugins)
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
        cmd_arch: Option<Arch>,
        cmd_debug: Option<bool>,
        cmd_uuid_path: Option<PathBuf>,
        common_env: Vec<(String, String)>,
        common_no_default_features: bool,
        common_features: Option<String>,
        cmd_optee_client_export: Option<PathBuf>,
        plugin: bool,
    ) -> Result<Self> {
        let component_type = if plugin {
            ComponentType::Plugin
        } else {
            ComponentType::Ca
        };

        // Get base configuration from metadata
        let metadata_config = MetadataConfig::resolve(project_path, component_type, cmd_arch)?;

        // Determine final arch: CLI > metadata > default
        let arch = cmd_arch
            .or_else(|| metadata_config.as_ref().map(|c| c.arch))
            .unwrap_or(Arch::Aarch64);

        // Handle priority: CLI > metadata > default
        let debug = cmd_debug
            .or_else(|| metadata_config.as_ref().map(|c| c.debug))
            .unwrap_or(false);

        // Handle optee_client_export: CLI > metadata > error (required)
        let optee_client_export_config = cmd_optee_client_export
            .or_else(|| {
                metadata_config
                    .as_ref()
                    .and_then(|c| c.optee_client_export.clone())
            })
            .ok_or_else(|| optee_client_export_error())?;

        // Resolve optee_client_export path (relative to absolute)
        let optee_client_export = resolve_path_relative_to_project(
            &optee_client_export_config,
            project_path,
            PathType::Directory,
            "OP-TEE client export directory",
        )?;

        // Handle uuid_path: only for plugins, CLI > metadata > default
        let uuid_path = if plugin {
            Some(resolve_uuid_path(
                cmd_uuid_path,
                metadata_config.as_ref().and_then(|c| c.uuid_path.clone()),
                project_path,
                PathBuf::from("../uuid.txt"),
            )?)
        } else {
            None
        };

        // Merge environment variables: metadata env + CLI env (CLI overrides metadata)
        let mut env = metadata_config
            .as_ref()
            .map(|c| c.env.clone())
            .unwrap_or_default();
        env.extend(common_env);

        Ok(CaBuildConfig {
            arch,
            debug,
            path: project_path.to_path_buf(),
            uuid_path,
            env,
            no_default_features: common_no_default_features,
            features: common_features,
            optee_client_export,
            plugin,
        })
    }

    /// Print the final CA/Plugin configuration parameters being used
    pub fn print_config(&self) {
        let component_name = if self.plugin { "Plugin" } else { "CA" };
        println!("Building {} with:", component_name);
        println!("  Arch: {:?}", self.arch);
        println!("  Debug: {}", self.debug);
        println!("  OP-TEE client export: {:?}", self.optee_client_export);
        if self.plugin {
            if let Some(ref uuid_path) = self.uuid_path {
                let absolute_uuid_path = uuid_path
                    .canonicalize()
                    .unwrap_or_else(|_| uuid_path.clone());
                println!("  UUID path: {:?}", absolute_uuid_path);
            }
        }
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
    pub uuid_path: Option<PathBuf>,
    /// additional environment key-value pairs, that should be passed to underlying
    /// build commands
    pub env: Vec<(String, String)>,
}

impl MetadataConfig {
    /// Extract build configuration from metadata only
    /// This function only parses metadata and does not handle priority resolution
    /// Determines arch with priority: cmd_arch > metadata > default
    /// Returns None if metadata is not found or parsing fails
    pub fn resolve(
        project_path: &Path,
        component_type: ComponentType,
        cmd_arch: Option<Arch>,
    ) -> Result<Option<Self>> {
        // Try to find application metadata (optional)
        let app_metadata = match discover_app_metadata(project_path) {
            Ok(meta) => meta,
            Err(_) => return Ok(None),
        };

        // Determine architecture with priority: cmd_arch > metadata > default
        let arch = cmd_arch
            .or_else(|| {
                app_metadata
                    .get("optee")?
                    .get(component_type.as_str())?
                    .get("arch")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or(Arch::Aarch64);

        // Extract metadata config with the determined architecture
        // Return None if metadata parsing fails (metadata not found or invalid)
        extract_build_config_with_arch(&app_metadata, arch, component_type)
            .map(Some)
            .or_else(|_| Ok(None))
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

/// Extract build configuration from application package metadata with specific architecture
fn extract_build_config_with_arch(
    metadata: &Value,
    arch: Arch,
    component_type: ComponentType,
) -> Result<MetadataConfig> {
    let optee_metadata = metadata
        .get("optee")
        .ok_or_else(|| anyhow::anyhow!("No optee metadata found in application package"))?;

    let component_metadata = optee_metadata.get(component_type.as_str()).ok_or_else(|| {
        anyhow::anyhow!(
            "No {} metadata found in optee section",
            component_type.as_str()
        )
    })?;

    // Parse debug with fallback to false
    let debug = component_metadata
        .get("debug")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Parse std with fallback to false
    let std = component_metadata
        .get("std")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

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
                    if v.is_string() {
                        v.as_str()
                    } else {
                        None
                    }
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
                        if v.is_string() {
                            v.as_str()
                        } else {
                            None
                        }
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

    // Parse uuid_path from metadata (for TA and Plugin)
    // component_metadata already points to the correct section (optee.ta or optee.plugin)
    let uuid_path =
        if component_type == ComponentType::Ta || component_type == ComponentType::Plugin {
            component_metadata
                .get("uuid-path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        } else {
            None // CA doesn't need uuid_path
        };

    Ok(MetadataConfig {
        arch,
        debug,
        std,
        ta_dev_kit_dir,
        optee_client_export,
        signing_key,
        uuid_path,
        env,
    })
}

/// Resolve uuid_path with priority: CLI > metadata > default
/// Returns the resolved absolute path
fn resolve_uuid_path(
    cmd_uuid_path: Option<PathBuf>,
    metadata_uuid_path: Option<PathBuf>,
    project_path: &Path,
    default: PathBuf,
) -> Result<PathBuf> {
    let uuid_path_was_from_cli = cmd_uuid_path.is_some();
    let uuid_path_str = cmd_uuid_path.or(metadata_uuid_path).unwrap_or(default);

    if uuid_path_was_from_cli {
        // CLI provided - resolve relative to current directory
        Ok(std::env::current_dir()?.join(&uuid_path_str))
    } else {
        // From metadata or default - resolve relative to project directory
        Ok(project_path.join(&uuid_path_str))
    }
}

/// Generate error message for missing ta-dev-kit-dir configuration
fn ta_dev_kit_dir_error() -> anyhow::Error {
    anyhow::anyhow!(
        "ta-dev-kit-dir is MANDATORY but not configured.\n\
        Please set it via:\n\
        1. Command line: --ta-dev-kit-dir <path>\n\
        2. Cargo.toml metadata: [package.metadata.optee.ta] section\n\
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
