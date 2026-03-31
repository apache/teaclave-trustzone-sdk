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
    if !cfg!(feature = "no_link") {
        link();
    }
    Ok(())
}

fn link() {
    let ta_dev_kit_dir = env::var("TA_DEV_KIT_DIR").expect("TA_DEV_KIT_DIR not set");
    let library_path = PathBuf::from(ta_dev_kit_dir).join("lib");
    if !library_path.exists() {
        panic!(
            "TA_DEV_KIT_DIR lib path {} does not exist",
            library_path.display()
        );
    }

    println!("cargo:rustc-link-search={}", library_path.display());
    println!("cargo:rustc-link-lib=static=utee");
    println!("cargo:rustc-link-lib=static=utils");
    println!("cargo:rustc-link-lib=static=mbedtls");
}
