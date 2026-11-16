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

use core::time::Duration;
use optee_utee::prelude::*;
use optee_utee::Time;
use optee_utee::{ErrorKind, Result};
use proto::time::Command;

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
fn invoke_command(cmd_id: u32, _params: &mut ParametersNone) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Test => {
            time()?;
            Ok(())
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

fn time() -> Result<()> {
    let ree_time = Time::ree_now();
    trace_println!("[+] Get REE time {}.", ree_time);
    trace_println!("[+] Now wait 1 second in TEE ...");
    Time::wait(Some(Duration::from_secs(1)))?;

    let system_time = Time::now();
    trace_println!("[+] Get system time {}.", system_time);

    let ta_time = system_time
        .checked_add(Duration::from_secs(5))
        .ok_or(ErrorKind::Overflow)?;
    ta_time.set_ta_time()?;

    let new_ta_time = Time::ta_now()?;
    trace_println!(
        "[+] After set the TA time 5 seconds ahead of system time, new TA time {}.",
        new_ta_time
    );
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
