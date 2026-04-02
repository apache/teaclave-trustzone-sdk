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
use super::{Setup, Socket, SocketAdapter, SocketError};

/// A trait used for convenience; import it so that the code remains consistent
/// with the std version (with the only difference being the return error type).
///
/// Take TcpStream as example:
/// ``` rust,no_run
/// use optee_utee::net::{Setup, TcpStream, SocketError};
///
/// fn connect_without_compact_trait(host: &str, port: u16) -> Result<TcpStream, SocketError> {
///     let setup = Setup::new_v4(host, port)?;
///     TcpStream::open(setup)
/// }
///
/// fn connect_with_compact_trait(host: &str, port: u16) -> Result<TcpStream, SocketError> {
///     use optee_utee::net::StdCompatConnect;
///
///     TcpStream::connect_v4(host, port)
/// }
/// ```
pub trait StdCompatConnect: Sized {
    fn connect_v4(address: &str, port: u16) -> Result<Self, SocketError>;
    fn connect_v6(address: &str, port: u16) -> Result<Self, SocketError>;
    fn connect(address: &str, port: u16) -> Result<Self, SocketError> {
        Self::connect_v4(address, port)
    }
}

/// A trait used for convenience; import it so that the code remains consistent
/// with the std version (with the only difference being the return error type).
///
/// Take TcpStream as example:
/// ``` rust,no_run
/// use optee_utee::net::{TcpStream, SocketError};
///
/// fn write_without_compact_trait(stream: &mut TcpStream, mut buf: &[u8]) -> Result<(), SocketError> {
///     use optee_utee::ErrorKind;
///     
///     while !buf.is_empty() {
///         match stream.send(buf) {
///             Ok(0) => return Err(SocketError::Tee(ErrorKind::Generic)),
///             Ok(n) => buf = &buf[n..],
///             Err(e) => return Err(e),
///         }
///     }
///     Ok(())
/// }
///
/// fn write_with_compact_trait(stream: &mut TcpStream, buf: &[u8]) -> Result<(), SocketError> {
///     use optee_utee::net::StdCompatWrite;
///
///     stream.write_all(buf)
/// }
/// ```
pub trait StdCompatWrite {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError>;
    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), SocketError> {
        while !buf.is_empty() {
            match self.write(buf)? {
                0 => return Err(SocketError::Tee(crate::ErrorKind::Generic)),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }
}

/// A trait used for convenience; import it so that the code remains consistent
/// with the std version (with the only difference being the return error type).
///
/// Take TcpStream as example:
/// ``` rust,no_run
/// use optee_utee::net::{TcpStream, SocketError};
///
/// fn read_without_compact_trait(stream: &mut TcpStream, mut buf: &mut [u8]) -> Result<(), SocketError> {
///     use optee_utee::ErrorKind;
///
///     while !buf.is_empty() {
///         match stream.recv(buf) {
///             Ok(0) => break,
///             Ok(n) => buf = &mut buf[n..],
///             Err(e) => return Err(e),
///         }
///     }
///     if !buf.is_empty() {
///         return Err(SocketError::Tee(ErrorKind::Generic));
///     }
///     Ok(())
/// }
///
/// fn read_with_compact_trait(stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), SocketError> {
///     use optee_utee::net::StdCompatRead;
///
///     stream.read_exact(buf)
/// }
/// ```
pub trait StdCompatRead {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError>;
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), SocketError> {
        while !buf.is_empty() {
            match self.read(buf)? {
                0 => break,
                n => buf = &mut buf[n..],
            }
        }
        if !buf.is_empty() {
            return Err(SocketError::Tee(crate::ErrorKind::Generic));
        }
        Ok(())
    }
}

impl<T: SocketAdapter<Setup = Setup>> StdCompatConnect for Socket<T> {
    fn connect_v4(address: &str, port: u16) -> Result<Self, SocketError> {
        let setup = Setup::new_v4(address, port)?;
        Self::open(setup)
    }
    fn connect_v6(address: &str, port: u16) -> Result<Self, SocketError> {
        let setup = Setup::new_v6(address, port)?;
        Self::open(setup)
    }
}

impl<T: SocketAdapter<Setup = Setup>> StdCompatWrite for Socket<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.send(buf)
    }
}

impl<T: SocketAdapter<Setup = Setup>> StdCompatRead for Socket<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        self.recv(buf)
    }
}
