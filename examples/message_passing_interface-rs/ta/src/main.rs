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

#![no_main]

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::{self, Command};
use std::io::Write;

fn handle_invoke(command: Command, input: proto::EnclaveInput) -> Result<proto::EnclaveOutput> {
    match command {
        Command::Hello => {
            let output = proto::EnclaveOutput {
                message: format!("Hello, {}", input.message),
            };
            Ok(output)
        }
        Command::Bye => {
            let output = proto::EnclaveOutput {
                message: format!("Bye, {}", input.message),
            };
            Ok(output)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

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
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_value()? };

    let input: proto::EnclaveInput = proto::serde_json::from_slice(p0.buffer()).map_err(|e| {
        trace_println!("Failed to deserialize input: {}", e);
        ErrorKind::BadFormat
    })?;
    let output = handle_invoke(Command::from(cmd_id), input)?;

    let output_vec = proto::serde_json::to_vec(&output).map_err(|e| {
        trace_println!("Failed to serialize output: {}", e);
        ErrorKind::BadFormat
    })?;
    p1.buffer().write_all(&output_vec).map_err(|e| {
        trace_println!("Failed to write output: {}", e);
        ErrorKind::BadParameters
    })?;
    p2.set_a(output_vec.len() as u32);

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
