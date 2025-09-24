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

use alloc::ffi::CString;
use core::ptr;
use optee_utee_sys as raw;

use super::{Socket, SocketAdapter, SocketError};

/// A setup parameter used for OP-TEE.
pub struct Setup {
    addr: CString,
    port: u16,
    version: raw::TEE_ipSocket_ipVersion,
}

impl Setup {
    pub(crate) fn new(
        addr: &str,
        port: u16,
        version: raw::TEE_ipSocket_ipVersion,
    ) -> crate::Result<Self> {
        Ok(Self {
            addr: CString::new(addr).map_err(|_| crate::ErrorKind::BadParameters)?,
            port,
            version,
        })
    }
    /// Construct a new IPv4 target parameter using the address and port. It
    /// will return `BadParameters` if the address contains a `\0` character in 
    /// the middle.
    pub fn new_v4(addr: &str, port: u16) -> crate::Result<Self> {
        Self::new(addr, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_4)
    }
    /// Construct a new IPv6 target parameter using the address and port. It
    /// will return `BadParameters` if the address contains a `\0` character in 
    /// the middle.
    pub fn new_v6(addr: &str, port: u16) -> crate::Result<Self> {
        Self::new(addr, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_6)
    }
}

/// An adapter for TCP sockets in OP-TEE. Typically, it is not used directly, 
/// but can be employed for wrapper operations, such as traffic control within 
/// the TEE.
pub struct TcpAdapter(raw::TEE_iSocketHandle);
/// An adapter for UDP sockets in OP-TEE. Typically, it is not used directly, 
/// but can be employed for wrapper operations, such as traffic control within 
/// the TEE.
pub struct UdpAdapter(raw::TEE_iSocketHandle);
/// A TcpStream that is compatible with OP-TEE.
pub type TcpStream = Socket<TcpAdapter>;
/// A UdpSocket that is compatible with OP-TEE.
pub type UdpSocket = Socket<UdpAdapter>;

fn handle_socket_operation_error(handle: raw::TEE_iSocketHandle, code: u32) -> SocketError {
    match code {
        raw::TEE_ISOCKET_ERROR_PROTOCOL => {
            let protocol_error = unsafe { ((*raw::TEE_tcpSocket).error)(handle) };
            SocketError::ErrorProtocol(protocol_error)
        }
        raw::TEE_ISOCKET_WARNING_PROTOCOL => {
            let protocol_error = unsafe { ((*raw::TEE_tcpSocket).error)(handle) };
            SocketError::WarningProtocol(protocol_error)
        }
        _ => SocketError::from_raw_error(code, 0),
    }
}

impl SocketAdapter for TcpAdapter {
    type Setup = Setup;
    type Handle = Self;

    fn open(setup: Self::Setup) -> Result<Self::Handle, SocketError> {
        let mut handle: raw::TEE_iSocketHandle = ptr::null_mut();
        let mut protocol_error: u32 = 0;
        let mut setup = raw::TEE_tcpSocket_Setup {
            ipVersion: setup.version,
            server_addr: setup.addr.as_ptr() as _,
            server_port: setup.port,
        };
        let ret = unsafe {
            ((*raw::TEE_tcpSocket).open)(
                &mut handle,
                &mut setup as *mut raw::TEE_tcpSocket_Setup as _,
                &mut protocol_error,
            )
        };
        match ret {
            raw::TEE_SUCCESS => Ok(Self(handle)),
            _ => Err(SocketError::from_raw_error(ret, protocol_error)),
        }
    }
    fn send(handle: &mut Self::Handle, buf: &[u8], timeout: u32) -> Result<usize, SocketError> {
        let mut length: u32 = buf.len() as _;
        let ret = unsafe {
            ((*raw::TEE_tcpSocket).send)(
                handle.0,
                buf.as_ptr() as _,
                &mut length,
                timeout,
            )
        };
        match ret {
            raw::TEE_SUCCESS => Ok(length as usize),
            _ => Err(handle_socket_operation_error(handle.0, ret)),
        }
    }
    fn recv(handle: &mut Self::Handle, buf: &mut [u8], timeout: u32) -> Result<usize, SocketError> {
        let mut length: u32 = buf.len() as _;
        let ret = unsafe {
            ((*raw::TEE_tcpSocket).recv)(handle.0, buf.as_mut_ptr() as _, &mut length, timeout)
        };
        match ret {
            raw::TEE_SUCCESS => Ok(length as usize),
            _ => Err(handle_socket_operation_error(handle.0, ret)),
        }
    }
}

impl Drop for TcpAdapter {
    fn drop(&mut self) {
        // Ignore any errors on close.
        unsafe {
            ((*raw::TEE_tcpSocket).close)(self.0);
        }
    }
}

impl SocketAdapter for UdpAdapter {
    type Setup = Setup;
    type Handle = Self;

    fn open(setup: Self::Setup) -> Result<Self::Handle, SocketError> {
        let mut handle: raw::TEE_iSocketHandle = ptr::null_mut();
        let mut protocol_error: u32 = 0;
        let mut setup = raw::TEE_udpSocket_Setup {
            ipVersion: setup.version,
            server_addr: setup.addr.as_ptr() as _,
            server_port: setup.port,
        };
        let ret = unsafe {
            ((*raw::TEE_udpSocket).open)(
                &mut handle,
                &mut setup as *mut raw::TEE_udpSocket_Setup as _,
                &mut protocol_error,
            )
        };
        match ret {
            raw::TEE_SUCCESS => Ok(Self(handle)),
            _ => Err(SocketError::from_raw_error(ret, protocol_error)),
        }
    }
    fn send(handle: &mut Self::Handle, buf: &[u8], timeout: u32) -> Result<usize, SocketError> {
        let mut length: u32 = buf.len() as _;
        let ret = unsafe {
            ((*raw::TEE_udpSocket).send)(
                handle.0,
                buf.as_ptr() as _,
                &mut length,
                timeout,
            )
        };
        match ret {
            raw::TEE_SUCCESS => Ok(length as usize),
            _ => Err(handle_socket_operation_error(handle.0, ret)),
        }
    }
    fn recv(handle: &mut Self::Handle, buf: &mut [u8], timeout: u32) -> Result<usize, SocketError> {
        let mut length: u32 = buf.len() as _;
        let ret = unsafe {
            ((*raw::TEE_udpSocket).recv)(handle.0, buf.as_mut_ptr() as _, &mut length, timeout)
        };
        match ret {
            raw::TEE_SUCCESS => Ok(length as usize),
            _ => Err(handle_socket_operation_error(handle.0, ret)),
        }
    }
}

impl Drop for UdpAdapter {
    fn drop(&mut self) {
        // Ignore any errors on close.
        unsafe {
            ((*raw::TEE_udpSocket).close)(self.0);
        }
    }
}
