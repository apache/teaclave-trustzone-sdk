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

use super::optee::Setup;
use super::SocketError;
use super::{TcpStream, UdpSocket};
use alloc::format;

impl TcpStream {
    fn connect_with_ip_version(
        address: &str,
        port: u16,
        ip_version: raw::TEE_ipSocket_ipVersion,
    ) -> std::io::Result<Self> {
        let setup = Setup::new(address, port, ip_version)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"))?;
        Ok(Self::open(setup).map_err(|err| Into::<std::io::Error>::into(err))?)
    }
    pub fn connect_v4(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_with_ip_version(address, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_4)
    }
    pub fn connect_v6(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_with_ip_version(address, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_6)
    }
    pub fn connect(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_v4(address, port)
    }
}

impl std::io::Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.recv(buf) {
            Ok(v) => Ok(v as usize),
            Err(err) => Err(Into::<std::io::Error>::into(err)),
        }
    }
}

impl std::io::Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.send(buf) {
            Ok(v) => Ok(v as usize),
            Err(err) => Err(Into::<std::io::Error>::into(err)),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl UdpSocket {
    fn connect_with_ip_version(
        address: &str,
        port: u16,
        ip_version: raw::TEE_ipSocket_ipVersion,
    ) -> std::io::Result<Self> {
        let setup = Setup::new(address, port, ip_version)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid address"))?;
        Ok(Self::open(setup).map_err(|err| Into::<std::io::Error>::into(err))?)
    }
    pub fn connect_v4(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_with_ip_version(address, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_4)
    }
    pub fn connect_v6(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_with_ip_version(address, port, raw::TEE_ipSocket_ipVersion::TEE_IP_VERSION_6)
    }
    pub fn connect(address: &str, port: u16) -> std::io::Result<Self> {
        Self::connect_v4(address, port)
    }
}

impl std::io::Read for UdpSocket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.recv(buf) {
            Ok(v) => Ok(v as usize),
            Err(err) => Err(Into::<std::io::Error>::into(err)),
        }
    }
}

impl std::io::Write for UdpSocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.send(buf) {
            Ok(v) => Ok(v as usize),
            Err(err) => Err(Into::<std::io::Error>::into(err)),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// This is implemented to save developers from having to make numerous map_err
// calls.
impl Into<std::io::Error> for SocketError {
    fn into(self) -> std::io::Error {
        use std::io::{Error, ErrorKind};
        match self {
            SocketError::ErrorProtocol(protocol_error) => Error::new(
                ErrorKind::Other,
                format!("TEE_ISOCKET_ERROR_PROTOCOL: 0x{:08X}", protocol_error),
            ),
            SocketError::RemoteClosed => Error::new(
                ErrorKind::ConnectionAborted,
                "TEE_ISOCKET_ERROR_REMOTE_CLOSED",
            ),
            SocketError::Timeout => Error::new(ErrorKind::TimedOut, "TEE_ISOCKET_ERROR_TIMEOUT"),
            SocketError::OutOfResource => {
                Error::new(ErrorKind::Other, "TEE_ISOCKET_ERROR_OUT_OF_RESOURCES")
            }
            SocketError::LargeBuffer => {
                Error::new(ErrorKind::Other, "TEE_ISOCKET_ERROR_LARGE_BUFFER")
            }
            SocketError::WarningProtocol(protocol_error) => Error::new(
                ErrorKind::Other,
                format!("TEE_ISOCKET_WARNING_PROTOCOL: 0x{:08X}", protocol_error),
            ),
            SocketError::Hostname => Error::new(ErrorKind::Other, "TEE_ISOCKET_ERROR_HOSTNAME"),
            SocketError::Tee(kind) => match kind {
                crate::ErrorKind::OutOfMemory => {
                    Error::new(ErrorKind::OutOfMemory, "TEE_ERROR_OUT_OF_MEMORY")
                }
                _ => Error::new(ErrorKind::Other, kind.as_str()),
            },
            SocketError::Unknown(code) => {
                Error::new(ErrorKind::Other, format!("Unknown: {:08X}", code))
            }
        }
    }
}
