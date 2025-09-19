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
use alloc::string::ToString;

use optee_utee::property::{
    ClientIdentity, PropertyKey, TaDescription, TaMultiSession, TeeInternalCoreVersion,
};
use optee_utee::LoginType;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;

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

fn get_properties() -> Result<()> {
    // check caller
    let client_identity = ClientIdentity.get()?;
    trace_println!(
        "[+] TA get caller identity: login: {}, uuid: {}",
        client_identity.login_type(),
        client_identity.uuid()
    );
    // login type should be Public
    if client_identity.login_type() != LoginType::Public {
        return Err(ErrorKind::BadParameters.into());
    }
    // uuid should be all zero
    if client_identity.uuid().to_string() != "00000000-0000-0000-0000-000000000000" {
        return Err(ErrorKind::BadParameters.into());
    }

    // test the other property:
    let core_version = TeeInternalCoreVersion.get()?;
    trace_println!("[+] TA get core version: {}", core_version);
    // core version should not be zero
    if core_version == 0 {
        return Err(ErrorKind::BadParameters.into());
    }

    let ta_multi_session = TaMultiSession.get()?;
    trace_println!("[+] TA get multi session: {}", ta_multi_session);
    // multi session should be false
    if ta_multi_session {
        return Err(ErrorKind::BadParameters.into());
    }

    let ta_description = TaDescription.get()?;
    trace_println!("[+] TA get description: {}", ta_description);
    // description should be the specified string
    if ta_description != "An example of Rust OP-TEE TrustZone SDK." {
        return Err(ErrorKind::BadParameters.into());
    }

    Ok(())
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Test => {
            get_properties()?;

            trace_println!("[+] Test passed");
            Ok(())
        }
        _ => Err(ErrorKind::NotSupported.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
