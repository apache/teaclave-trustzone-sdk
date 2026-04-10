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

use optee_utee_build::{Error, TaConfig};

fn main() -> Result<(), Error> {
    // For Rust editions 2018 and earlier, You must set workspace.resolver = "2"
    // in your Cargo.toml to prevent feature unification of optee-utee-sys when
    // it is used in both dependencies and build-dependencies.
    //
    // For editions after 2018, this setting is enabled by default and does not
    // need to be specified.
    //
    // For reference:
    // 1. resolver version 2: https://doc.rust-lang.org/cargo/reference/resolver.html#feature-resolver-version-2
    // 2. resolver versions: https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions
    let flags: u32 = optee_utee_sys::TA_FLAG_SINGLE_INSTANCE
        | optee_utee_sys::TA_FLAG_MULTI_SESSION
        | optee_utee_sys::TA_FLAG_INSTANCE_KEEP_ALIVE;

    let config = TaConfig::new_default_with_cargo_env(proto::UUID)?.ta_flags(flags);
    optee_utee_build::build(config)
}
