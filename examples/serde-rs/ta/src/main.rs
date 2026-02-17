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

use alloc::string::String;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::{Command, Point};

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
        Command::DefaultOp => {
            let mut p = unsafe { params.0.as_memref()? }.output()?;
            let mut buffer = p.buffer()?;
            let point = Point { x: 1, y: 2 };

            // Convert the Point to a JSON string.
            let serialized: String = serde_json::to_string(&point).map_err(|e| {
                trace_println!("Failed to serialize point: {}", e);
                ErrorKind::BadParameters
            })?;

            let bytes = serialized.as_bytes();

            // Ensure the buffer is large enough to hold the serialized data.
            let len = bytes.len();
            p.ensure_capacity(len)?;

            // Copy the serialized JSON string into the buffer.
            buffer.copy_from(bytes)?;

            // update size of output buffer
            if p.set_updated_size(len).is_err() {
                unreachable!("already made sure the buffer has enough capacity")
            }

            // Prints serialized = {"x":1,"y":2}
            trace_println!("serialized = {}", serialized);

            // Convert the JSON string back to a Point.
            let deserialized: Point = serde_json::from_str(&serialized).map_err(|e| {
                trace_println!("Failed to deserialize point: {}", e);
                ErrorKind::BadParameters
            })?;

            // Prints deserialized = Point { x: 1, y: 2 }
            trace_println!("deserialized = {:?}", deserialized);

            Ok(())
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
