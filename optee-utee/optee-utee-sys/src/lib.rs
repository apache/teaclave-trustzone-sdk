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
#![allow(non_camel_case_types, non_snake_case)]

pub use tee_api::api::*;
pub use tee_api_defines::*;
pub use tee_api_types::*;
pub use tee_internal_api_extensions::api::*;
pub use tee_ipsocket::*;
pub use tee_isocket::*;
pub use tee_tcpsocket::*;
pub use tee_udpsocket::*;
pub use trace::*;
pub use user_ta_header::*;
pub use utee_syscalls::*;
pub use utee_types::*;

mod tee_api;
mod tee_api_defines;
mod tee_api_types;
mod tee_internal_api_extensions;
mod tee_ipsocket;
mod tee_isocket;
mod tee_tcpsocket;
mod tee_udpsocket;
mod trace;
mod user_ta_header;
mod utee_syscalls;
mod utee_types;

// Currently, the libc crate does not support optee_os, and patching it in
// Xargo.toml within the TA project does not affect optee-utee-sys. Therefore,
// we need to define the type directly in the crate to ensure compatibility.
#[cfg(feature = "std")]
mod libc_compat {
    pub type size_t = usize;
    pub type intmax_t = i64;
}

#[cfg(not(feature = "std"))]
mod libc_compat {
    pub use libc::{intmax_t, size_t};
}

#[cfg(feature = "mock")]
pub mod mock_api {
    pub use crate::tee_api::mock_api::*;
    pub mod extension {
        pub use crate::tee_internal_api_extensions::mock_api::*;
    }
}

#[cfg(feature = "mock")]
pub mod mock_utils;
