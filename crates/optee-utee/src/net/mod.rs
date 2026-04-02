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

mod error;
mod optee;
mod socket;

pub use error::SocketError;
pub use optee::{Setup, TcpAdapter, TcpStream, UdpAdapter, UdpSocket};
pub use socket::{Socket, SocketAdapter};

#[cfg(feature = "std")]
mod optee_std;

mod optee_no_std;
pub use optee_no_std::{StdCompatConnect, StdCompatRead, StdCompatWrite};
