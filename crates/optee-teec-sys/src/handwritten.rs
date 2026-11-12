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

//! Declarations that bindgen cannot generate from the C headers.
//!
//! `generated` is produced by
//! `cargo run --manifest-path xtask/Cargo.toml -- generate-bindings teec` from
//! the OP-TEE client headers listed in `HEADERS` in
//! `xtask/src/generate_bindings/teec/generate.rs`. An item belongs here only
//! when bindgen structurally cannot express it.

use crate::generated::*;
use core::ffi::c_ulong;

// Function-like macros from `tee_client_api.h`:
//
//     #define TEEC_PARAM_TYPES(p0, p1, p2, p3) \
//         ((p0) | ((p1) << 4) | ((p2) << 8) | ((p3) << 12))
//     #define TEEC_PARAM_TYPE_GET(p, i) (((p) >> (i * 4)) & 0xF)
//
// bindgen does not expand function-like macros, so they are reimplemented as
// ordinary functions with the same arithmetic. The index is a `usize` so callers
// can pass array indices without casts.

/// The four four-bit parameter types packed into one value by
/// [`TEEC_PARAM_TYPES`].
///
/// `tee_client_api.h` has no typedef for this: the parameter types are plain
/// `uint32_t` macros, so the alias only gives the packed value a name.
pub type TEEC_ParamType = u32;

pub fn TEEC_PARAM_TYPES(
    p0: TEEC_ParamType,
    p1: TEEC_ParamType,
    p2: TEEC_ParamType,
    p3: TEEC_ParamType,
) -> TEEC_ParamType {
    p0 | p1 << 4 | p2 << 8 | p3 << 12
}

pub fn TEEC_PARAM_TYPE_GET(p: TEEC_ParamType, i: usize) -> TEEC_ParamType {
    (p >> (i * 4)) & 0xF
}

/// `ULONG_MAX` from `tee_client_api.h`.
///
/// Even with the clang macro fallback bindgen emits nothing for this one, so it
/// is hand-written. `c_ulong` keeps the value correct on both 32-bit and 64-bit
/// hosts instead of pinning the width of the target used for generation.
pub const TEEC_CONFIG_SHAREDMEM_MAX_SIZE: c_ulong = c_ulong::MAX;

/// `tee_plugin_method.h` declares `struct plugin_method`; the SDK has always
/// exposed it under this name.
pub type PluginMethod = plugin_method;
