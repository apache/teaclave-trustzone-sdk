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

use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use std::thread;

use optee_teec::{Context, ErrorKind, Operation, ParamType, Session, Uuid};
use optee_teec::{ParamNone, ParamTmpRef, ParamValue};
use proto::{Command, IpVersion, UUID};

fn tcp_client(
    session: &mut Session,
    address: &str,
    port: u16,
    ip_version: IpVersion,
    host_name: &str,
) -> optee_teec::Result<()> {
    println!("Test on: {}", address);

    let http_data = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host_name);
    let mut operation = Operation::new(
        0,
        ParamTmpRef::new_input(address.as_bytes()),
        ParamValue::new(port as u32, ip_version as u32, ParamType::ValueInput),
        ParamTmpRef::new_input(http_data.as_bytes()),
        ParamNone,
    );
    session.invoke_command(Command::Start as u32, &mut operation)?;

    println!("Success");
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    // test ipv4
    const IPV4_HOST: &str = "teaclave.apache.org";
    // Use the host directly to also check its domain name resolving capability.
    tcp_client(&mut session, IPV4_HOST, 80, IpVersion::V4, IPV4_HOST)?;

    // test ipv6
    let addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0));
    let server = tiny_http::Server::http(addr).map_err(|e| {
        eprintln!("Failed to create HTTP server: {}", e);
        ErrorKind::BadParameters
    })?;
    let listen_addr = server.server_addr().to_ip().ok_or_else(|| {
        eprintln!("Failed to get server IP address");
        ErrorKind::BadParameters
    })?;
    let ip = listen_addr.ip().to_string();
    let port = listen_addr.port();

    let child = thread::spawn(move || {
        if let Some(request) = server.incoming_requests().next() {
            println!(
                "received request! method: {:?}, url: {:?}, headers: {:?}",
                request.method(),
                request.url(),
                request.headers()
            );

            let response = tiny_http::Response::from_string("hello world");
            if let Err(e) = request.respond(response) {
                eprintln!("Failed to respond to request: {}", e);
            }
        }
    });
    // Use the IP address directly to ensure we're actually trying an IPv6
    // address.
    tcp_client(&mut session, &ip, port, IpVersion::V6, &ip)?;
    let _ = child.join();

    Ok(())
}
