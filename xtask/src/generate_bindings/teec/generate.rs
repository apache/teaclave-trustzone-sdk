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

//! bindgen pass over the OP-TEE client headers.

use std::error::Error;
use std::path::Path;

/// Headers from the OP-TEE client export's `usr/include/` directory.
///
/// `tee_plugin_method.h` includes `tee_client_api.h`, so these two cover the
/// whole client ABI.
const HEADERS: &[&str] = &["tee_client_api.h", "tee_plugin_method.h"];

fn bindgen_builder(include_dir: &Path, target: &str) -> bindgen::Builder {
    let mut builder = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("--target={target}"))
        .clang_arg("-ffreestanding")
        .use_core()
        .wrap_unsafe_ops(true)
        .formatter(bindgen::Formatter::Prettyplease);

    for header in HEADERS {
        builder = builder.header(include_dir.join(header).display().to_string());
    }

    builder
}

/// Generates the bindings.
///
/// Types, constants and functions all come from one bindgen run: unlike the TA
/// side, the client crate has no `mock` feature that would need the functions
/// wrapped in a module.
pub(super) fn generate(include_dir: &Path, target: &str) -> Result<String, Box<dyn Error>> {
    let bindings = bindgen_builder(include_dir, target)
        .allowlist_type("TEEC_.*")
        .allowlist_type("plugin_method")
        .allowlist_var("TEEC_.*")
        .allowlist_var("TEE_ERROR_.*")
        .allowlist_function("TEEC_.*")
        // Error constants are aliases (`TEEC_ERROR_TARGET_DEAD` ->
        // `TEE_ERROR_TARGET_DEAD`) and `TEEC_CONFIG_SHAREDMEM_MAX_SIZE` is
        // `ULONG_MAX`, neither of which bindgen's token parser can evaluate.
        .clang_macro_fallback()
        // See the `utee` generator: the types are target independent, and the
        // assertions would tie the output to the machine that ran bindgen.
        .layout_tests(false)
        .generate()?;
    Ok(bindings.to_string())
}
