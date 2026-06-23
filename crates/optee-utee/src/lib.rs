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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc, doc = concat!(
    env!("CARGO_PKG_DESCRIPTION"),
    "\n",
    "## Feature flags\n",
    document_features::document_features!(),
))]

// Requires `alloc`.
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
use libc_alloc::LibcAlloc;

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(all(not(feature = "std"), not(feature = "no_panic_handler")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        optee_utee_sys::TEE_Panic(0);
    }
    loop {}
}

// Stubs for unwinding symbols required by the precompiled sysroot when not using
// `-Z build-std`. These are never actually called because we compile with
// `-C panic=abort`, but the linker requires the symbols to be present.
#[cfg(all(not(feature = "std"), feature = "unwind_stubs"))]
mod unwind_stubs {
    #[unsafe(no_mangle)]
    extern "C" fn _Unwind_Resume() -> ! {
        loop {}
    }

    #[unsafe(no_mangle)]
    extern "C" fn rust_eh_personality() {}
}

pub use arithmetical::*;
pub use crypto_op::*;
pub use error::{Error, ErrorKind, Result};
pub use extension::*;
pub use identity::{Identity, LoginType};
pub use object::*;
pub use optee_utee_macros::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session,
};
pub use parameter::{
    FromRawParameter, FromRawParameters, ParamType, ParameterAny, ParametersAny, ParametersNone,
    RawParamType, RawParamTypes, RawParams, deprecated,
    memref::{
        ParameterMemrefInout, ParameterMemrefInput, ParameterMemrefOutput, ParameterMemrefRead,
        ParameterMemrefWrite,
    },
    none::ParameterNone,
    value::{
        ParameterValueInout, ParameterValueInput, ParameterValueOutput, ParameterValueRead,
        ParameterValueWrite,
    },
};
pub use ta_session::{TaSession, TaSessionBuilder};
pub use tee_parameter::{ParamIndex, TeeParams};
pub use time::*;
pub use uuid::*;

pub mod trace;
#[macro_use]
mod macros;
pub mod arithmetical;
pub mod crypto_op;
mod error;
pub mod extension;
pub mod identity;
pub mod net;
pub mod object;
mod parameter;
pub mod property;
mod ta_session;
mod tee_parameter;
pub mod time;
pub mod uuid;

// Re-export optee_utee_sys so developers don't have to add it to their cargo
// dependencies.
pub use optee_utee_sys as raw;

pub mod prelude {
    pub use crate::{
        FromRawParameter, FromRawParameters, ParameterAny, ParameterMemrefInout,
        ParameterMemrefInput, ParameterMemrefOutput, ParameterMemrefRead, ParameterMemrefWrite,
        ParameterNone, ParameterValueInout, ParameterValueInput, ParameterValueOutput,
        ParameterValueRead, ParameterValueWrite, ParametersAny, ParametersNone, ta_close_session,
        ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_print, trace_println,
    };
}
