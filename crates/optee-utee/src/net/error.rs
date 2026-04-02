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

use optee_utee_sys as raw;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SocketError {
    ErrorProtocol(u32),
    RemoteClosed,
    Timeout,
    OutOfResource,
    WarningProtocol(u32),
    LargeBuffer,
    Hostname,
    Tee(crate::ErrorKind),
    Unknown(u32),
}

impl SocketError {
    pub fn from_raw_error(code: u32, protocol_error: u32) -> Self {
        match code {
            raw::TEE_ISOCKET_ERROR_PROTOCOL => Self::ErrorProtocol(protocol_error),
            raw::TEE_ISOCKET_ERROR_REMOTE_CLOSED => Self::RemoteClosed,
            raw::TEE_ISOCKET_ERROR_TIMEOUT => Self::Timeout,
            raw::TEE_ISOCKET_ERROR_OUT_OF_RESOURCES => Self::OutOfResource,
            raw::TEE_ISOCKET_ERROR_LARGE_BUFFER => Self::LargeBuffer,
            raw::TEE_ISOCKET_WARNING_PROTOCOL => Self::WarningProtocol(protocol_error),
            raw::TEE_ISOCKET_ERROR_HOSTNAME => Self::Hostname,
            raw::TEE_ERROR_CANCEL
            | raw::TEE_ERROR_COMMUNICATION
            | raw::TEE_ERROR_OUT_OF_MEMORY
            | raw::TEE_ERROR_BAD_PARAMETERS => Self::Tee(crate::Error::from_raw_error(code).kind()),
            _ => Self::Unknown(code),
        }
    }
}

impl core::fmt::Display for SocketError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<crate::Error> for SocketError {
    fn from(value: crate::Error) -> Self {
        Self::Tee(value.kind())
    }
}
