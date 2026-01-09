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

use super::SocketError;
use core::time::Duration;

/// A trait designed to accommodate various implementations of GP TEE Sockets
/// API.
///
/// An implementation of this trait is responsible for handling all
/// protocol-related tasks, including but not limited to:
/// * Defining its own Setup type and using it to establish a new connection;
/// * Sending and receiving data over the connection, while managing protocol
/// errors (such as permitting certain warnings but raising others).
pub trait SocketAdapter: Sized {
    type Setup;
    type Handle;

    fn open(setup: Self::Setup) -> Result<Self::Handle, SocketError>;
    fn send(handle: &mut Self::Handle, buf: &[u8], timeout: u32) -> Result<usize, SocketError>;
    fn recv(handle: &mut Self::Handle, buf: &mut [u8], timeout: u32) -> Result<usize, SocketError>;
}

/// A struct used for socket operations.
pub struct Socket<T: SocketAdapter> {
    handle: T::Handle,
    recv_timeout: u32,
    send_timeout: u32,
}

impl<T: SocketAdapter> Socket<T> {
    /// create a new connection, and then sending and receiving data over it.
    pub fn open(setup: T::Setup) -> Result<Self, SocketError> {
        let handle = T::open(setup)?;
        Ok(Self {
            handle,
            recv_timeout: raw::TEE_TIMEOUT_INFINITE,
            send_timeout: raw::TEE_TIMEOUT_INFINITE,
        })
    }
    /// set timeout of recv operation.
    pub fn set_recv_timeout_in_milli(&mut self, milliseconds: u32) {
        self.recv_timeout = milliseconds;
    }
    /// set timeout of send operation.
    pub fn set_send_timeout_in_milli(&mut self, milliseconds: u32) {
        self.send_timeout = milliseconds;
    }
    /// a wrapper of `set_recv_timeout_in_milli`, similar to `set_read_timeout`
    /// in std::net::TcpStream, it will set timeout to `TEE_TIMEOUT_INFINITE`
    /// if `Option::None` is provided.
    pub fn set_recv_timeout(&mut self, dur: Option<Duration>) -> crate::Result<()> {
        let milliseconds = convert_duration_option_to_timeout(dur)?;
        self.set_recv_timeout_in_milli(milliseconds);
        Ok(())
    }
    /// a wrapper of `set_send_timeout_in_milli`, similar to
    /// `set_write_timeout` in std::net::TcpStream, it will set timeout to
    /// `TEE_TIMEOUT_INFINITE` if `Option::None` is provided.
    pub fn set_send_timeout(&mut self, dur: Option<Duration>) -> crate::Result<()> {
        let milliseconds = convert_duration_option_to_timeout(dur)?;
        self.set_send_timeout_in_milli(milliseconds);
        Ok(())
    }
    /// send data, similar to `write` in `io::Write`
    pub fn send(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        T::send(&mut self.handle, buf, self.send_timeout)
    }
    /// recv data, similar to `read` in `io::Read`
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        T::recv(&mut self.handle, buf, self.recv_timeout)
    }
}

fn convert_duration_option_to_timeout(dur: Option<Duration>) -> crate::Result<u32> {
    match dur {
        None => Ok(raw::TEE_TIMEOUT_INFINITE),
        Some(v) => {
            let milliseconds = v.as_millis();
            if milliseconds > (u32::MAX as u128) {
                return Err(crate::ErrorKind::BadParameters.into());
            }
            Ok(milliseconds as u32)
        }
    }
}
