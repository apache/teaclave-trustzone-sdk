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

#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

extern crate alloc;

use alloc::format;
use optee_utee::prelude::*;
use optee_utee::{ErrorKind, Result};
use proto::Command;

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
fn open_session(_params: &mut ParametersNone) -> Result<()> {
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
fn invoke_command(
    cmd_id: u32,
    (p0, p1, _, _): &mut (
        ParameterMemrefInput<'_>,
        ParameterMemrefOutput<'_>,
        ParameterNone,
        ParameterNone,
    ),
) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let input: proto::EnclaveInput = serde_json::from_slice(p0.get_buffer()).map_err(|e| {
        trace_println!("Failed to deserialize input: {}", e);
        ErrorKind::BadFormat
    })?;
    let output = handle_invoke(Command::from(cmd_id), input)?;

    let output_vec = serde_json::to_vec(&output).map_err(|e| {
        trace_println!("Failed to serialize output: {}", e);
        ErrorKind::BadFormat
    })?;
    p1.set_output(output_vec)?;

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
