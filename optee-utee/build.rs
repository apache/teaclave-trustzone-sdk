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

use std::fs;

/// Outputs unstable #[feature = "foo"] iff the rustc version is older than $version
macro_rules! maybe_feat {
    ($out:expr, $feat:literal, $version:literal) => {{
        let filename = $out.join(concat!($feat, ".rs"));

        let s = if version_check::is_min_version($version).unwrap_or(false) {
            String::new()
        } else {
            format!("#![feature({})]\n", $feat)
        };
        fs::write(filename, s).expect("failed to write to {filename:?}");
    }};
}

fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("infallible"));

    // The custom patched std version is currently on 1.80. When we upgrade, we should
    // bump the MSRV accordingly and remove any of these features that are stablized.
    maybe_feat!(out, "error_in_core", "1.81.0");
}
