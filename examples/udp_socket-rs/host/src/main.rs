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
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::thread;

use optee_teec::{Context, ErrorKind, Operation, Result, Uuid};
use optee_teec::{ParamNone, ParamTmpRef, ParamValue};
use proto::{Command, IpVersion, UUID};

fn udp_socket(ip_version: IpVersion) -> Result<()> {
    let addr: SocketAddr = match ip_version {
        IpVersion::V4 => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
        IpVersion::V6 => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0)),
    };
    let socket = UdpSocket::bind(addr).map_err(|e| {
        eprintln!("Failed to bind UDP socket: {}", e);
        ErrorKind::BadParameters
    })?;
    let local_addr = socket.local_addr().map_err(|e| {
        eprintln!("Failed to get UDP local address: {}", e);
        ErrorKind::BadParameters
    })?;
    println!("Test on: {}", local_addr);

    let child = thread::spawn(move || -> Result<()> {
        let mut ctx = Context::new()?;
        let uuid = Uuid::parse_str(UUID)?;
        let mut session = ctx.open_session(uuid)?;

        let ip = local_addr.ip().to_string();
        let port = local_addr.port();
        let mut operation = Operation::new(
            0,
            ParamTmpRef::new_input(ip.as_bytes()),
            ParamValue::new(
                port as u32,
                ip_version as u32,
                optee_teec::ParamType::ValueInput,
            ),
            ParamNone,
            ParamNone,
        );
        session.invoke_command(Command::Start as u32, &mut operation)?;
        Ok(())
    });

    let mut buf = [0; 100];
    let (_, src_addr) = socket.recv_from(&mut buf).map_err(|e| {
        eprintln!("UDP recv error: {}", e);
        ErrorKind::BadParameters
    })?;
    socket
        .send_to(b"[Host] Hello, Teaclave!", src_addr)
        .map_err(|e| {
            eprintln!("UDP send error: {}", e);
            ErrorKind::BadParameters
        })?;
    println!("{}", String::from_utf8_lossy(&buf));
    child.join().map_err(|_| {
        eprintln!("Thread panicked");
        ErrorKind::Generic
    })??;

    println!("Success");
    Ok(())
}

fn main() -> Result<()> {
    udp_socket(IpVersion::V4)?;
    udp_socket(IpVersion::V6)?;

    Ok(())
}
