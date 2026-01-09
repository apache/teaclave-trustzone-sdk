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

pub type TEE_tcpSocket_Setup = TEE_tcpSocket_Setup_s;
#[repr(C)]
pub struct TEE_tcpSocket_Setup_s {
    pub ipVersion: TEE_ipSocket_ipVersion,
    pub server_addr: *mut c_char,
    pub server_port: u16,
}

extern "C" {
    pub static TEE_tcpSocket: *const TEE_iSocket;
}

pub const TEE_ISOCKET_PROTOCOLID_TCP: u32 = 0x65;
pub const TEE_ISOCKET_TCP_WARNING_UNKNOWN_OUT_OF_BAND: u32 = 0xF1010002;

pub const TEE_TCP_SET_RECVBUF: u32 = 0x65f00000;
pub const TEE_TCP_SET_SENDBUF: u32 = 0x65f00001;
