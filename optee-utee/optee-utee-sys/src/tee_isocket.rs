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

use super::*;
use core::ffi::*;

pub type TEE_iSocketHandle = *mut c_void;
pub type TEE_iSocket = TEE_iSocket_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TEE_iSocket_s {
    pub TEE_iSocketVersion: u32,
    pub protocolID: u8,
    pub open: unsafe extern "C" fn(
        ctx: *mut TEE_iSocketHandle,
        setup: *mut c_void,
        protocolError: *mut u32,
    ) -> TEE_Result,
    pub close: unsafe extern "C" fn(ctx: TEE_iSocketHandle) -> TEE_Result,
    pub send: unsafe extern "C" fn(
        ctx: TEE_iSocketHandle,
        buf: *const c_void,
        length: *mut u32,
        timeout: u32,
    ) -> TEE_Result,
    pub recv: unsafe extern "C" fn(
        ctx: TEE_iSocketHandle,
        buf: *mut c_void,
        length: *mut u32,
        timeout: u32,
    ) -> TEE_Result,
    pub error: unsafe extern "C" fn(ctx: TEE_iSocketHandle) -> u32,
    pub ioctl: unsafe extern "C" fn(
        ctx: TEE_iSocketHandle,
        commandCode: u32,
        buf: *mut c_void,
        length: *mut u32,
    ) -> TEE_Result,
}

pub const TEE_ISOCKET_VERSION: u32 = 0x01000000;

pub const TEE_ISOCKET_ERROR_PROTOCOL: u32 = 0xF1007001;
pub const TEE_ISOCKET_ERROR_REMOTE_CLOSED: u32 = 0xF1007002;
pub const TEE_ISOCKET_ERROR_TIMEOUT: u32 = 0xF1007003;
pub const TEE_ISOCKET_ERROR_OUT_OF_RESOURCES: u32 = 0xF1007004;
pub const TEE_ISOCKET_ERROR_LARGE_BUFFER: u32 = 0xF1007005;
pub const TEE_ISOCKET_WARNING_PROTOCOL: u32 = 0xF1007006;
pub const TEE_ISOCKET_ERROR_HOSTNAME: u32 = 0xF1007007;
