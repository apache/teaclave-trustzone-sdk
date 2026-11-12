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
//! `cargo run --manifest-path xtask/Cargo.toml -- generate-bindings utee` from
//! the OP-TEE TA dev kit headers listed in `HEADERS` in
//! `xtask/src/generate_bindings/utee/generate.rs`. An item belongs here only
//! when bindgen structurally cannot express it:
//!
//! * macros that take arguments (`TEE_PARAM_TYPES`, `TEE_PARAM_TYPE_GET`);
//! * macros whose value is an integer cast to a pointer (`TEE_PROPSET_*`);
//! * definitions or non-exported declarations (`__utee_entry`, `__ta_entry`).
//!
//! Everything else must stay generated. The command above only rewrites
//! `generated.rs`, so re-read the items below against the headers when bumping
//! `optee-version.txt`.

use crate::generated::*;
use core::ffi::c_ulong;

// `tee_api_defines.h` defines the pseudo property-set handles as pointer casts:
//
//     #define TEE_PROPSET_TEE_IMPLEMENTATION (TEE_PropSetHandle)0xFFFFFFFD
//
// and `TEE_PropSetHandle` is an opaque pointer (`*mut __TEE_PropSetHandle`).
// bindgen cannot fold an integer cast to a pointer type into a Rust constant and
// emits nothing for these macros, even with `clang_macro_fallback`. They are
// therefore hand-written.
//
// These values are sentinel pointers rather than real addresses: libutee and the
// TEE core compare them by value and never dereference them, so they must match
// the C definitions exactly. `TEE_PROPSET_TEE_IMPLEMENTATION` is the only one
// callers normally pass to `TEE_GetProperty*`; the other two exist because the
// header defines them.
pub const TEE_PROPSET_TEE_IMPLEMENTATION: TEE_PropSetHandle = 0xFFFFFFFD as *mut _;
pub const TEE_PROPSET_CURRENT_CLIENT: TEE_PropSetHandle = 0xFFFFFFFE as *mut _;
pub const TEE_PROPSET_CURRENT_TA: TEE_PropSetHandle = 0xFFFFFFFF as *mut _;

/// The four four-bit parameter types packed into one value by
/// [`TEE_PARAM_TYPES`].
///
/// `tee_api_defines.h` has no typedef for this: the parameter types are plain
/// `uint32_t` macros, so the alias only gives the packed value a name.
pub type TEE_ParamType = u32;

// Function-like macros from `tee_api_defines.h`:
//
//     #define TEE_PARAM_TYPES(t0,t1,t2,t3) \
//         ((t0) | ((t1) << 4) | ((t2) << 8) | ((t3) << 12))
//     #define TEE_PARAM_TYPE_GET(t, i) ((((uint32_t)t) >> ((i)*4)) & 0xF)
//
// bindgen does not expand function-like macros, so they are reimplemented as
// ordinary functions with the same arithmetic. The index is a `usize` so callers
// can pass array indices without casts.
pub fn TEE_PARAM_TYPES(
    t0: TEE_ParamType,
    t1: TEE_ParamType,
    t2: TEE_ParamType,
    t3: TEE_ParamType,
) -> TEE_ParamType {
    t0 | t1 << 4 | t2 << 8 | t3 << 12
}

pub fn TEE_PARAM_TYPE_GET(t: TEE_ParamType, i: usize) -> TEE_ParamType {
    (t >> (i * 4)) & 0xF
}

// libutee's entry point, called by `__ta_entry` below. It is declared in the TA
// dev kit's `src/user_ta_header.c`, which is not part of the exported `include/`
// tree, so bindgen cannot see it and the declaration has to live here.
//
// Together with `__ta_entry` this is a frozen ABI: the TEE core, libutee and
// every TA on the device agree on it, so a change would be a breaking change for
// OP-TEE itself. Neither declaration is checked against the dev kit at build
// time for that reason.
unsafe extern "C" {
    fn __utee_entry(
        func: c_ulong,
        session_id: c_ulong,
        up: *mut utee_params,
        cmd_id: c_ulong,
    ) -> TEE_Result;
}

/// The TA entry point, `__ta_entry`.
///
/// This symbol cannot be generated because it is a *definition*, not a
/// declaration, and it lives in `TA_DEV_KIT_DIR/src/user_ta_header.c`, which is
/// not part of the exported `include/` tree bindgen reads. The TA must provide
/// the symbol itself: the dev kit's `mk/link.mk` links TAs with `-e__ta_entry`,
/// and `crates/optee-utee-build/src/linker.rs` re-emits that flag for Rust TAs,
/// so without this function the TA would not link.
///
/// The body mirrors the C implementation: forward to libutee's `__utee_entry`
/// and hand the result to `_utee_return`.
///
/// * On ARM32 the C file compiles its definition as `__ta_entry_c` and gets the
///   `__ta_entry` alias from `ta/arch/arm/ta_entry_a32.S`, purely to stop
///   unwinding. A Rust TA defines `__ta_entry` directly on both targets, which
///   is the name the linker is asked for, and needs no unwinding shim.
/// * `_utee_return` is declared `-> !`, hence this function never returns.
/// * The C version also initialises the stack canary and calls `ftrace_return()`
///   under `CFG_TA_STACK_PROTECTOR` / `CFG_FTRACE_SUPPORT`. Rust TAs are not
///   built with those C options, so the trampoline stays minimal.
///
/// # Safety
/// This function is the main entry point for a Trusted Application (TA) in
/// OP-TEE. It must only be called by the OP-TEE OS with valid parameters. The
/// `up` parameter is a raw pointer that must point to a valid `utee_params`
/// structure initialized by the OP-TEE runtime environment. This function
/// should never be called directly from user code - it is only exported for the
/// OP-TEE OS loader.
#[unsafe(no_mangle)]
unsafe fn __ta_entry(
    func: c_ulong,
    session_id: c_ulong,
    up: *mut utee_params,
    cmd_id: c_ulong,
) -> ! {
    let res: TEE_Result = unsafe { __utee_entry(func, session_id, up, cmd_id) };

    unsafe { _utee_return(res.into()) };
}
