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

/// Build configuration that can be discovered from proto metadata
#[derive(Debug, Clone)]
pub struct BuildConfig {
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

impl BuildConfig {
    /// Create a new build config by resolving parameters with priority:
    /// 1. Command line arguments (highest priority)
    /// 2. [package.metadata.optee.<component_type>] in Cargo.toml
    /// 3. Default values or error for mandatory parameters
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        project_path: &Path,
        component_type: &str, // "ta", "ca", or "plugin"
        cmd_arch: Option<Arch>,
        cmd_debug: Option<bool>,
        cmd_std: Option<bool>,
        cmd_ta_dev_kit_dir: Option<PathBuf>,
        cmd_optee_client_export: Option<PathBuf>,
        cmd_signing_key: Option<PathBuf>,
        cmd_uuid_path: Option<PathBuf>,
    ) -> Result<Self> {
        // Try to find application metadata (optional)
        let app_metadata = discover_app_metadata(project_path).ok();

        // Try to get metadata config if available
        let metadata_config = app_metadata
            .as_ref()
            .and_then(|meta| extract_build_config_from_metadata(meta, component_type).ok());

        // Resolve architecture with priority: CLI > metadata > default
        let arch = cmd_arch
            .or_else(|| metadata_config.as_ref().map(|m| m.arch))
            .unwrap_or(Arch::Aarch64);

        // Re-resolve metadata with the final architecture if it was overridden
        let final_metadata_config = if let Some(ref app_meta) = app_metadata {
            if cmd_arch.is_some() && cmd_arch != metadata_config.as_ref().map(|m| m.arch) {
                extract_build_config_with_arch(app_meta, arch, component_type).ok()
            } else {
                metadata_config
            }
        } else {
            None
        };

        // Resolve parameters with priority: CLI > metadata > default
        let debug = cmd_debug
            .or_else(|| final_metadata_config.as_ref().map(|m| m.debug))
            .unwrap_or(false);

        let std = cmd_std
            .or_else(|| final_metadata_config.as_ref().map(|m| m.std))
            .unwrap_or(false);

        // Resolve library paths with priority: CLI > metadata > None
        let ta_dev_kit_dir = cmd_ta_dev_kit_dir.or_else(|| {
            final_metadata_config
                .as_ref()
                .and_then(|m| m.ta_dev_kit_dir.clone())
        });

        let optee_client_export = cmd_optee_client_export.or_else(|| {
            final_metadata_config
                .as_ref()
                .and_then(|m| m.optee_client_export.clone())
        });

        let signing_key = cmd_signing_key.or_else(|| {
            final_metadata_config
                .as_ref()
                .and_then(|m| m.signing_key.clone())
        });

        // Resolve uuid_path with priority: CLI > Cargo.toml metadata > default (../uuid.txt)
        let uuid_path = cmd_uuid_path
            .or_else(|| {
                // Try to read uuid_path from package metadata
                app_metadata
                    .as_ref()
                    .and_then(|meta| extract_uuid_path_from_metadata(meta).ok())
            })
            .unwrap_or_else(|| PathBuf::from("../uuid.txt"));

        Ok(BuildConfig {
            arch,
            debug,
            std,
            ta_dev_kit_dir,
            optee_client_export,
            signing_key,
            uuid_path: Some(uuid_path),
            env: final_metadata_config
                .as_ref()
                .map(|m| m.env.clone())
                .unwrap_or_default(),
        })
    }

    /// Print the final configuration parameters being used
    pub fn print_config(&self, component_type: &str, project_path: &Path) {
        println!("Building {} with:", component_type.to_uppercase());
        println!("  Arch: {:?}", self.arch);
        println!("  Debug: {}", self.debug);

        if component_type == "ta" {
            println!("  Std: {}", self.std);
            if let Some(ref ta_dev_kit_dir) = self.ta_dev_kit_dir {
                let absolute_ta_dev_kit_dir = if ta_dev_kit_dir.is_absolute() {
                    ta_dev_kit_dir.clone()
                } else {
                    project_path.join(ta_dev_kit_dir)
                };
                println!("  TA dev kit dir: {:?}", absolute_ta_dev_kit_dir);
            }
            if let Some(ref signing_key) = self.signing_key {
                let absolute_signing_key = if signing_key.is_absolute() {
                    signing_key.clone()
                } else {
                    project_path.join(signing_key)
                };
                println!("  Signing key: {:?}", absolute_signing_key);
            }
            if let Some(ref uuid_path) = self.uuid_path {
                let absolute_uuid_path = project_path
                    .join(uuid_path)
                    .canonicalize()
                    .unwrap_or_else(|_| project_path.join(uuid_path));
                println!("  UUID path: {:?}", absolute_uuid_path);
            }
        }

        if component_type == "ca" || component_type == "plugin" {
            if let Some(ref optee_client_export) = self.optee_client_export {
                let absolute_optee_client_export = if optee_client_export.is_absolute() {
                    optee_client_export.clone()
                } else {
                    project_path.join(optee_client_export)
                };
                println!("  OP-TEE client export: {:?}", absolute_optee_client_export);
            }
            if component_type == "plugin" {
                if let Some(ref uuid_path) = self.uuid_path {
                    let absolute_uuid_path = project_path
                        .join(uuid_path)
                        .canonicalize()
                        .unwrap_or_else(|_| project_path.join(uuid_path));
                    println!("  UUID path: {:?}", absolute_uuid_path);
                }
            }
        }
        if !self.env.is_empty() {
            println!("  Environment variables: {} set", self.env.len());
        }
    }

    /// Get required ta_dev_kit_dir or return error
    pub fn require_ta_dev_kit_dir(&self) -> Result<PathBuf> {
        self.ta_dev_kit_dir
            .clone()
            .ok_or_else(|| anyhow::anyhow!(
                "ta-dev-kit-dir is MANDATORY but not configured.\n\
                Please set it via:\n\
                1. Command line: --ta-dev-kit-dir <path>\n\
                2. Cargo.toml metadata: [package.metadata.optee.ta] section\n\
                \n\
                Example Cargo.toml:\n\
                [package.metadata.optee.ta]\n\
                ta-dev-kit-dir = {{ aarch64 = \"/path/to/optee_os/out/arm-plat-vexpress/export-ta_arm64\" }}\n\
                # arm architecture omitted (defaults to null)\n\
                \n\
                For help with available options, run: cargo-optee build ta --help"
            ))
    }

    /// Get required optee_client_export or return error
    pub fn require_optee_client_export(&self) -> Result<PathBuf> {
        self.optee_client_export
            .clone()
            .ok_or_else(|| anyhow::anyhow!(
                "optee-client-export is MANDATORY but not configured.\n\
                Please set it via:\n\
                1. Command line: --optee-client-export <path>\n\
                2. Cargo.toml metadata: [package.metadata.optee.ca] or [package.metadata.optee.plugin] section\n\
                \n\
                Example Cargo.toml:\n\
                [package.metadata.optee.ca]\n\
                optee-client-export = {{ aarch64 = \"/path/to/optee_client/export_arm64\" }}\n\
                # arm architecture omitted (defaults to null)\n\
                \n\
                For help with available options, run: cargo-optee build ca --help"
            ))
    }

    /// Get uuid_path (defaults to "../uuid.txt" if not specified)
    pub fn get_uuid_path(&self) -> PathBuf {
        self.uuid_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("../uuid.txt"))
    }

    /// Get signing key with fallback to default
    pub fn resolve_signing_key(&self, ta_dev_kit_dir: &Path) -> PathBuf {
        self.signing_key
            .clone()
            .unwrap_or_else(|| ta_dev_kit_dir.join("keys").join("default_ta.pem"))
    }
}

/// Extract UUID path from package metadata
fn extract_uuid_path_from_metadata(metadata: &Value) -> Result<PathBuf> {
    // Try to get optee.ta.uuid-path from metadata
    if let Some(optee_metadata) = metadata.get("optee") {
        if let Some(ta_section) = optee_metadata.get("ta") {
            if let Some(uuid_path_value) = ta_section.get("uuid-path") {
                if let Some(uuid_path_str) = uuid_path_value.as_str() {
                    return Ok(PathBuf::from(uuid_path_str));
                }
            }
        }
        // Also try plugin section for plugin builds
        if let Some(plugin_section) = optee_metadata.get("plugin") {
            if let Some(uuid_path_value) = plugin_section.get("uuid-path") {
                if let Some(uuid_path_str) = uuid_path_value.as_str() {
                    return Ok(PathBuf::from(uuid_path_str));
                }
            }
        }
    }

    // Default fallback
    Err(anyhow::anyhow!("No uuid_path found in metadata"))
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

/// Extract build configuration from application package metadata
fn extract_build_config_from_metadata(
    metadata: &Value,
    component_type: &str,
) -> Result<BuildConfig> {
    let optee_metadata = metadata
        .get("optee")
        .ok_or_else(|| anyhow::anyhow!("No optee metadata found in application package"))?;

    let component_metadata = optee_metadata
        .get(component_type)
        .ok_or_else(|| anyhow::anyhow!("No {} metadata found in optee section", component_type))?;

    // Parse arch with fallback to default
    let arch = component_metadata
        .get("arch")
        .and_then(|v| v.as_str())
        .unwrap_or("aarch64")
        .parse()
        .unwrap_or(Arch::Aarch64);

    extract_build_config_with_arch(metadata, arch, component_type)
}

/// Extract build configuration from application package metadata with specific architecture
fn extract_build_config_with_arch(
    metadata: &Value,
    arch: Arch,
    component_type: &str,
) -> Result<BuildConfig> {
    let optee_metadata = metadata
        .get("optee")
        .ok_or_else(|| anyhow::anyhow!("No optee metadata found in application package"))?;

    let component_metadata = optee_metadata
        .get(component_type)
        .ok_or_else(|| anyhow::anyhow!("No {} metadata found in optee section", component_type))?;

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
    let ta_dev_kit_dir = if component_type == "ta" {
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
    let optee_client_export = if component_type == "ca" || component_type == "plugin" {
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
    let signing_key = if component_type == "ta" {
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

    Ok(BuildConfig {
        arch,
        debug,
        std,
        ta_dev_kit_dir,
        optee_client_export,
        signing_key,
        uuid_path: None, // Not extracted from metadata, handled separately
        env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_build_config_from_metadata() {
        let metadata = json!({
            "optee": {
                "ta": {
                    "arch": "arm",
                    "debug": true,
                    "std": true,
                    "ta-dev-kit-dir": {
                        "aarch64": "/opt/ta_dev_kit_arm64",
                        "arm": "/opt/ta_dev_kit_arm32"
                    },
                    "signing-key": "/opt/signing.pem",
                    "env": [
                        "RUSTFLAGS=-C target-feature=+crt-static",
                        "RUST_LOG=debug"
                    ]
                }
            }
        });

        let config = extract_build_config_from_metadata(&metadata, "ta").unwrap();
        assert!(matches!(config.arch, Arch::Arm));
        assert!(config.debug);
        assert!(config.std);
        assert_eq!(
            config.ta_dev_kit_dir,
            Some(PathBuf::from("/opt/ta_dev_kit_arm32"))
        );
        assert_eq!(config.optee_client_export, None); // Not for TA
        assert_eq!(config.signing_key, Some(PathBuf::from("/opt/signing.pem")));
        assert_eq!(config.env.len(), 2);
        assert!(config.env.contains(&(
            "RUSTFLAGS".to_string(),
            "-C target-feature=+crt-static".to_string()
        )));
        assert!(config
            .env
            .contains(&("RUST_LOG".to_string(), "debug".to_string())));
    }

    #[test]
    fn test_extract_build_config_with_arch_override() {
        let metadata = json!({
            "optee": {
                "ca": {
                    "arch": "arm",
                    "debug": false,
                    "optee-client-export": {
                        "aarch64": "/opt/client_arm64",
                        "arm": "/opt/client_arm32"
                    },
                    "env": [
                        "BUILD_MODE=release"
                    ]
                }
            }
        });

        let config = extract_build_config_with_arch(&metadata, Arch::Aarch64, "ca").unwrap();
        assert!(matches!(config.arch, Arch::Aarch64));
        assert!(!config.debug);
        assert!(!config.std);
        assert_eq!(config.ta_dev_kit_dir, None); // Not for CA
        assert_eq!(
            config.optee_client_export,
            Some(PathBuf::from("/opt/client_arm64"))
        );
        assert_eq!(config.signing_key, None); // Not for CA
        assert_eq!(config.env.len(), 1);
        assert!(config
            .env
            .contains(&("BUILD_MODE".to_string(), "release".to_string())));
    }

    #[test]
    fn test_extract_build_config_defaults() {
        let metadata = json!({
            "optee": {
                "plugin": {}
            }
        });

        let config = extract_build_config_from_metadata(&metadata, "plugin").unwrap();
        assert!(matches!(config.arch, Arch::Aarch64));
        assert!(!config.debug);
        assert!(!config.std);
        assert_eq!(config.ta_dev_kit_dir, None);
        assert_eq!(config.optee_client_export, None); // Not for Plugin
        assert_eq!(config.signing_key, None); // Not for Plugin
        assert!(config.env.is_empty());
    }

    #[test]
    fn test_extract_build_config_with_env_variables() {
        let metadata = json!({
            "optee": {
                "ta": {
                    "env": [
                        "CUSTOM_VAR=value1",
                        "ANOTHER_VAR=value2",
                        "RUSTFLAGS=-C target-cpu=native"
                    ]
                }
            }
        });

        let config = extract_build_config_from_metadata(&metadata, "ta").unwrap();
        assert_eq!(config.env.len(), 3);
        assert!(config
            .env
            .contains(&("CUSTOM_VAR".to_string(), "value1".to_string())));
        assert!(config
            .env
            .contains(&("ANOTHER_VAR".to_string(), "value2".to_string())));
        assert!(config
            .env
            .contains(&("RUSTFLAGS".to_string(), "-C target-cpu=native".to_string())));
    }

    #[test]
    fn test_extract_build_config_with_invalid_env_format() {
        let metadata = json!({
            "optee": {
                "ca": {
                    "env": [
                        "VALID_VAR=value",
                        "INVALID_VAR_NO_EQUALS",
                        "ANOTHER_VALID=test"
                    ]
                }
            }
        });

        let config = extract_build_config_from_metadata(&metadata, "ca").unwrap();
        // Should only contain the valid environment variables
        assert_eq!(config.env.len(), 2);
        assert!(config
            .env
            .contains(&("VALID_VAR".to_string(), "value".to_string())));
        assert!(config
            .env
            .contains(&("ANOTHER_VALID".to_string(), "test".to_string())));
        // Invalid format should be filtered out
        assert!(!config.env.iter().any(|(k, _)| k == "INVALID_VAR_NO_EQUALS"));
    }

    #[test]
    fn test_extract_build_config_with_missing_arch() {
        let metadata = json!({
            "optee": {
                "ta": {
                    "arch": "aarch64",
                    "ta-dev-kit-dir": {
                        "aarch64": "/opt/ta_dev_kit_arm64"
                        // arm key missing - should be treated as null
                    },
                    "signing-key": "/opt/signing.pem"
                }
            }
        });

        // Test with aarch64 - should get the path
        let config = extract_build_config_with_arch(&metadata, Arch::Aarch64, "ta").unwrap();
        assert_eq!(
            config.ta_dev_kit_dir,
            Some(PathBuf::from("/opt/ta_dev_kit_arm64"))
        );

        // Test with arm - should get None due to missing key (treated as null)
        let config_arm = extract_build_config_with_arch(&metadata, Arch::Arm, "ta").unwrap();
        assert_eq!(config_arm.ta_dev_kit_dir, None);
    }
}
