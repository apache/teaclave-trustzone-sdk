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

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result, Uuid};
use optee_utee::{ParamIndex, TaSessionBuilder, TeeParams};
use proto::{
    Command, HelloWorldTaCommand, SystemPtaCommand, HELLO_WORLD_USER_TA_UUID, SYSTEM_PTA_UUID,
};

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

fn test_invoke_system_pta() -> Result<()> {
    let system_pta_uuid = Uuid::parse_str(SYSTEM_PTA_UUID)?;
    // Open a session using the default timeout (TEE_TIMEOUT_INFINITE, meaning no timeout), and no parameters:
    let mut session = TaSessionBuilder::new(system_pta_uuid).build()?;
    trace_println!("[+] TA open PTA session success");

    let input: [u8; 32] = [0; 32];
    let mut output: [u8; 32] = [0; 32];

    // Construct parameters using chained method calls, max 4 parameters:
    let mut params = TeeParams::new()
        .with_memref_in(ParamIndex::Arg0, &input)
        .with_memref_out(ParamIndex::Arg1, &mut output);

    // Invoke the command using the default timeout, meaning no timeout:
    session.invoke_command(SystemPtaCommand::DeriveTaUniqueKey as u32, &mut params)?;

    // Get the output buffer through written_slice():
    // Note: written_slice() returns a slice of the original output buffer, with the length
    // adjusted to the actual size written by the TA.
    let written_slice = params[ParamIndex::Arg1]
        .written_slice()
        .ok_or(ErrorKind::BadParameters)?;

    trace_println!("[+] TA invoke PTA command, output: {:?}", written_slice);

    // Check if the output is not all zeroes and has the expected length:
    if written_slice.len() != 32 {
        trace_println!(
            "[-] TA invoke PTA command failed, wrong output length: {:?}",
            written_slice.len()
        );
        return Err(ErrorKind::Generic.into());
    }

    if written_slice.iter().all(|&x| x == 0) {
        trace_println!("[-] TA invoke PTA command failed, output is all 0");
        return Err(ErrorKind::Generic.into());
    }

    trace_println!("[+] TA invoke System PTA command success");

    Ok(())
}

fn test_invoke_hello_world_user_ta() -> Result<()> {
    let hello_world_user_ta_uuid = Uuid::parse_str(HELLO_WORLD_USER_TA_UUID)?;
    // Open a session with a specified timeout in milliseconds (10 seconds).
    // To pass parameters during session opening, use `.with_params(xxx)`.
    let mut session = TaSessionBuilder::new(hello_world_user_ta_uuid)
        .with_timeout(10000)
        .build()?;
    trace_println!("[+] TA open user TA session success");

    let mut params = TeeParams::new().with_value_inout(ParamIndex::Arg0, 29, 0);
    // Invoke the command with a specified timeout in milliseconds (10 seconds).
    session.invoke_command_with_timeout(
        HelloWorldTaCommand::IncValue as u32,
        &mut params,
        10000,
    )?;

    // Get the output value pair through output_value():
    let (value_a, _value_b) = params[ParamIndex::Arg0]
        .output_value()
        .ok_or(ErrorKind::BadParameters)?;

    // Check if the output value is as expected:
    if value_a != 129 {
        trace_println!(
            "[-] TA invoke user TA command failed, wrong output value: {:?}",
            value_a
        );
        return Err(ErrorKind::Generic.into());
    }

    trace_println!("[+] TA invoke hello world TA command success.");

    Ok(())
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Test => {
            test_invoke_system_pta()?;
            test_invoke_hello_world_user_ta()?;

            trace_println!("[+] Test passed");
            Ok(())
        }
        _ => Err(ErrorKind::NotSupported.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
