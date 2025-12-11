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

const ENV_OPTEE_CLIENT_EXPORT: &str = "OPTEE_CLIENT_EXPORT";
const AARCH64: &str = "aarch64-unknown-linux-gnu";
const X86_64: &str = "x86_64-unknown-linux-gnu";

#[derive(Debug, Clone, Copy)]
enum Target {
    Aarch64,
    X86_64,
}

impl Target {
    fn get() -> Result<Target, String> {
        let var = env::var("TARGET").expect("infallible");
        match var.as_str() {
            AARCH64 => Ok(Target::Aarch64),
            X86_64 => Ok(Target::X86_64),
            _ => Err(var),
        }
    }
}

/// Lists out env vars to check in order of priority
fn enumerate_vars_for_target() -> Vec<String> {
    let prefix = ENV_OPTEE_CLIENT_EXPORT.to_owned();
    let Ok(target) = Target::get() else {
        // Unknown targets should not try using the suffixed variants.
        return vec![prefix];
    };
    let aarch64 = format!("_{AARCH64}");
    let x86 = format!("_{X86_64}");
    let suffixes = match target {
        Target::Aarch64 => &[&aarch64, &aarch64.replace("-", "_"), ""],
        Target::X86_64 => &[&x86, &x86.replace("-", "_"), ""],
    };

    suffixes
        .iter()
        .map(|s| format!("{prefix}{s}"))
        .collect()
}

fn main() -> Result<(), VarError> {
    if !is_feature_enable("no_link")? {
        link();
    }
    Ok(())
}

// Check if feature enabled.
// Refer to: https://doc.rust-lang.org/cargo/reference/features.html#build-scripts
fn is_feature_enable(feature: &str) -> Result<bool, VarError> {
    let feature_env = format!("CARGO_FEATURE_{}", feature.to_uppercase().replace("-", "_"));

    match env::var(feature_env) {
        Err(VarError::NotPresent) => Ok(false),
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}

fn link() {
    let vars_to_check = enumerate_vars_for_target();
    for var in vars_to_check.iter() {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let mut selected_env_var = None;
    for var in vars_to_check {
        match env::var(&var) {
            Ok(value) => {
                if value.trim().is_empty() {
                    continue;
                }
                selected_env_var = Some((var, value));
                break;
            }
            Err(VarError::NotUnicode(_)) => panic!("could not parse {} as unicode", var),
            Err(VarError::NotPresent) => continue,
        }
    }
    let (selected_env_var, optee_client_dir) = selected_env_var
        .expect("Neither OPTEE_CLIENT_EXPORT nor a target specific variant were set");

    let library_path = PathBuf::from(optee_client_dir).join("usr/lib");
    if !library_path.exists() {
        panic!(
            "{} usr/lib path {} does not exist",
            selected_env_var,
            library_path.display()
        );
    }

    println!("cargo:rustc-link-search={}", library_path.display());
    println!("cargo:rustc-link-lib=dylib=teec");
}
