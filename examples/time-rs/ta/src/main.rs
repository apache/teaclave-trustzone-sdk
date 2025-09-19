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

use optee_utee::Time;
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

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut Parameters) -> Result<()> {
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
    let mut time = Time::new();
    time.ree_time();
    trace_println!("[+] Get REE time {}.", time);
    trace_println!("[+] Now wait 1 second in TEE ...");
    Time::wait(1000)?;
    time.system_time();
    trace_println!("[+] Get system time {}.", time);
    time.seconds += 5;
    time.set_ta_time()?;
    let mut time2 = Time::new();
    time2.ta_time()?;
    trace_println!(
        "[+] After set the TA time 5 seconds ahead of system time, new TA time {}.",
        time2
    );
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
