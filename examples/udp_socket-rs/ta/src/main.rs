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

// Use feature flag to detect STD environment
// - Feature "std" means std is available
// - No feature means no-std environment
// Check `feature`, not `target_os`, to avoid:
// error: unexpected `cfg` value: `optee` reported by clippy
// This occurs because `optee` is not an upstream Rust target_os.
#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

cfg_block::cfg_block! {
    // In Teaclave, if feature "std" is enabled, the codes is compiled with std.
    // Otherwise, no-std
    if #[cfg(feature = "std")] {
        use std::io::{Read, Write};
    } else {
        extern crate alloc;
        use optee_utee::net::{StdCompatConnect, StdCompatWrite, StdCompatRead};
        use alloc::vec::Vec;
        use alloc::string::String;
    }
}

use core::convert::TryFrom;
use optee_utee::net::UdpSocket;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::{Command, IpVersion};

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Start => {
            let mut param0 = unsafe { params.0.as_memref()? };
            let param1 = unsafe { params.1.as_value()? };

            let address = core::str::from_utf8(param0.buffer()).map_err(|e| {
                trace_println!("Failed to parse address from UTF-8: {}", e);
                ErrorKind::BadParameters
            })?;
            let port = param1.a() as u16;
            let ip_version = IpVersion::try_from(param1.b()).map_err(|_| {
                trace_println!("Invalid IP version parameter");
                ErrorKind::BadParameters
            })?;

            udp_socket(address, port, ip_version)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

fn udp_socket(address: &str, port: u16, ip_version: IpVersion) -> Result<()> {
    let mut stream = match ip_version {
        IpVersion::V4 => UdpSocket::connect_v4(address, port),
        IpVersion::V6 => UdpSocket::connect_v6(address, port),
    }
    .map_err(|err| {
        trace_println!("Failed to connect to {}:{}: {}", address, port, err);
        ErrorKind::Generic
    })?;

    stream.set_send_timeout_in_milli(60 * 1000);
    stream.set_recv_timeout_in_milli(60 * 1000);

    stream.write_all(b"[TA]: Hello, Teaclave!").map_err(|err| {
        trace_println!("Failed to write data: {}", err);
        ErrorKind::Generic
    })?;
    let mut response = Vec::new();
    let mut chunk = [0u8; 1024];

    // Loop until read something.
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => continue,
            Ok(n) => {
                response.extend_from_slice(&chunk[..n]);
                break;
            }
            Err(err) => {
                trace_println!("Failed to read data: {}", err);
                return Err(ErrorKind::Generic.into());
            }
        }
    }
    trace_println!("{}", String::from_utf8_lossy(&response));
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
