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
    let optee_os_dir = env::var("TA_DEV_KIT_DIR").expect("TA_DEV_KIT_DIR not set");
    let library_path = PathBuf::from(optee_os_dir).join("lib");
    if !library_path.exists() {
        panic!(
            "Library path {} does not exist",
            library_path.display()
        );
    }

    println!("cargo:rustc-link-search={}", library_path.display());
    println!("cargo:rustc-link-lib=static=utee");
    println!("cargo:rustc-link-lib=static=utils");
    println!("cargo:rustc-link-lib=static=mbedtls");
}
