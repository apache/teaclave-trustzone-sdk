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

use std::{env, path::PathBuf};

fn main() {
    let mut cfg = ctest::TestGenerator::new();
    let path = {
        let mut tmp = PathBuf::from(env::var("OPTEE_CLIENT_EXPORT").unwrap());
        tmp.push("usr/include");
        tmp
    };
    cfg.language(ctest::Lang::C)
        .target("aarch64-unknown-linux-gnu")
        .header("tee_client_api.h")
        .include(path.display().to_string())
        .type_name(|s, _is_struct, _is_union| s.to_string())
        .field_name(|_s, field| {
            if field.starts_with("imp__") {
                return format!("imp.{}", field.strip_prefix("imp__").expect("must ok"));
            }
            field.to_string()
        })
        .skip_struct(|s| s.ends_with("__Imp"))
        // The roundtrip implementation in ctest doesn't work with nested
        // structs —it treats all bytes of TEEC_Session as if there’s no
        // padding, which causes a mismatch in the last 4 padding bytes
        // during the roundtrip test.
        .skip_roundtrip(|s| s == "TEEC_Session")
        .skip_field_type(|s, field| {
            (s == "TEEC_SharedMemory"
                || s == "TEEC_Context"
                || s == "TEEC_Session"
                || s == "TEEC_Operation")
                && field == "imp"
        });
    cfg.generate("../optee-teec-sys/src/lib.rs", "all.rs");
}
