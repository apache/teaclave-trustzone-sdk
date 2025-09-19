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

use alloc::{boxed::Box, string::String};
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
fn open_session(params: &mut Parameters, ctx: &mut String) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };

    let mut buffer = [0_u8; proto::IDENTITY_SIZE];
    optee_utee::Random::generate(&mut buffer);
    *ctx = hex::encode(buffer).to_uppercase();

    trace_println!("[+] TA open session, identity: {}", ctx);
    p0.buffer().copy_from_slice(&buffer);

    Ok(())
}

#[ta_close_session]
fn close_session(ctx: &mut String) {
    trace_println!("[+] TA close session, identity: {}", ctx);
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(ctx: &mut String, cmd_id: u32, params: &mut Parameters) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Sleep => {
            let values = unsafe { params.0.as_value()? };

            let milliseconds = values.a();
            let mut now = optee_utee::Time::new();
            now.system_time();
            // this would cause messy output in the console when running concurrently.
            trace_println!(
                "[+] TA {} wait for {} milliseconds at {}",
                ctx,
                milliseconds,
                now.seconds * 1000 + now.millis
            );
            optee_utee::Time::wait(milliseconds)
        }
        Command::Unknown => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
