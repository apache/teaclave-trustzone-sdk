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

use rustc_version::{version, Version};

fn main() {
    // fix lint
    println!("cargo::rustc-check-cfg=cfg(optee_utee_rustc_has_stable_error_in_core)");

    let v = version().expect("infailable");
    if v >= Version::new(1, 81, 0) {
        println!("cargo:rustc-cfg=optee_utee_rustc_has_stable_error_in_core");
    }
}
