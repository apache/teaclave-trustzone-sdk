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

use std::env::{self, VarError};
use std::path::PathBuf;

fn main() -> Result<(), VarError> {
    if !is_feature_enabled("no_link")? {
        link(is_env_present("TEEC_STATIC")?);
    }
    Ok(())
}

fn is_env_present(var: &str) -> Result<bool, VarError> {
    println!("cargo:rerun-if-env-changed={var}");
    match env::var(var) {
        Err(VarError::NotPresent) => Ok(false),
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}

/// Checks if feature is enabled.
/// Refer to: https://doc.rust-lang.org/cargo/reference/features.html#build-scripts
fn is_feature_enabled(feature: &str) -> Result<bool, VarError> {
    let feature_env = format!("CARGO_FEATURE_{}", feature.to_uppercase().replace("-", "_"));
    is_env_present(&feature_env)
}

fn link(static_linkage: bool) {
    const ENV_OPTEE_CLIENT_EXPORT: &str = "OPTEE_CLIENT_EXPORT";
    println!("cargo:rerun-if-env-changed={}", ENV_OPTEE_CLIENT_EXPORT);

    let optee_client_dir =
        env::var(ENV_OPTEE_CLIENT_EXPORT).expect("OPTEE_CLIENT_EXPORT is not set");
    let library_path = PathBuf::from(optee_client_dir).join("usr/lib");
    if !library_path.exists() {
        panic!(
            "OPTEE_CLIENT_EXPORT usr/lib path {} does not exist",
            library_path.display()
        );
    }

    println!("cargo:rustc-link-search={}", library_path.display());
    println!(
        "cargo:rustc-link-lib={}=teec",
        if static_linkage { "static" } else { "dylib" }
    );
}
