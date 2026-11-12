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

//! bindgen passes over the TA dev kit headers.

use std::error::Error;
use std::path::Path;

/// Headers from the OP-TEE TA dev kit `include/` directory.
///
/// Only the top-level headers are listed; everything else comes in through
/// their `#include`s. The count matters: bindgen's macro fallback names its
/// precompiled header after all inputs but the last, and with more than about a
/// dozen names the filename exceeds the filesystem limit, silently disabling
/// the fallback (and with it the `TEE_ALG_ECDSA_P*` / `TEE_ALG_ECDH_P*`
/// aliases).
const HEADERS: &[&str] = &[
    "tee_api.h",
    "tee_internal_api_extensions.h",
    "tee_isocket.h",
    "tee_tcpsocket.h",
    "tee_udpsocket.h",
    "user_ta_header.h",
    "utee_syscalls.h",
    "utee_defines.h",
    "trace.h",
    "__tee_tcpsocket_defines_extensions.h",
];

/// Creates the bindgen builder shared by every pass: the full header list, the
/// target triple, `no_std` (`use_core`), and the prettyplease formatter so the
/// committed output is stable.
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
/// Types, constants and the two used libutee syscalls come from one bindgen run.
/// The mockable API and trace functions are generated in a second run, wrapped in
/// a module so `#[automock]` can produce the crate's `mock_api`.
pub(super) fn generate(include_dir: &Path, target: &str) -> Result<String, Box<dyn Error>> {
    let bindings = bindgen_builder(include_dir, target)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        // Types.
        .allowlist_type("TEE_.*")
        .allowlist_type("utee_.*")
        .allowlist_type("ta_head")
        .allowlist_type("user_ta_.*")
        .allowlist_type("__TEE_.*")
        .allowlist_type("intmax_t")
        // Constants.
        .allowlist_var("TEE_.*")
        .allowlist_var("TA_FLAGS_MASK")
        .allowlist_var("TA_PROP_STR_.*")
        // `TA_FLAG_*` values are object-like macros whose bodies call `BIT32`;
        // clang resolves them through the macro fallback below.
        .allowlist_var("TA_FLAG_.*")
        // Only these two libutee syscalls are used by the Rust TA; the rest of
        // `utee_syscalls.h` is libutee's own business. `_utee_return` returns
        // `!`, so it must stay out of the mocked `api` module below.
        .allowlist_function("_utee_log|_utee_return")
        // Some deprecated constants are aliases of other macros, for example
        // `TEE_ALG_ECDSA_P192` -> `TEE_ALG_ECDSA_SHA1`. bindgen's token parser
        // cannot evaluate those, so let clang resolve them.
        .clang_macro_fallback()
        // Layout assertions only repeat the target's own ABI: the generated
        // types are the same on every target (C `size_t` is `usize`, `long` is
        // `c_long`, pointers are pointers) and `#[repr(C)]` does the rest.
        // Keeping them would make the output depend on the target used here.
        .layout_tests(false)
        .generate()?;

    let mut output = bindings.to_string();

    // All mockable functions live in one module; `#[automock]` generates the
    // crate's `mock_api` next to it.
    let functions = generate_functions(
        include_dir,
        target,
        &[
            "TEE_.*",
            "tee_invoke_supp_plugin",
            "trace_ext_puts",
            "trace_ext_get_thread_id",
            "trace_set_level",
            "trace_get_level",
            "trace_printf",
            "dhex_dump",
        ],
    )?;
    let api_module = [
        "",
        "#[allow(unused_imports)]",
        "#[cfg_attr(feature = \"mock\", mockall::automock)]",
        "pub mod api {",
        "    use super::*;",
        functions.as_str(),
        "}",
        "",
    ];
    output.push_str(&api_module.join("\n"));

    Ok(output)
}

fn generate_functions(
    include_dir: &Path,
    target: &str,
    allowlist: &[&str],
) -> Result<String, Box<dyn Error>> {
    let mut builder = bindgen_builder(include_dir, target)
        .blocklist_type(".*")
        .blocklist_var(".*");
    for pattern in allowlist {
        builder = builder.allowlist_function(pattern);
    }
    Ok(builder.generate()?.to_string())
}
