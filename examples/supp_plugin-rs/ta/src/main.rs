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

#![no_std]
#![no_main]

extern crate alloc;

use optee_utee::LoadablePlugin;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result, Uuid};
use proto::{Command, PluginCommand, PLUGIN_SUBCMD_NULL, PLUGIN_UUID};

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
    let inbuf = p0.buffer().to_vec();
    trace_println!(
        "[+] TA received value {:?} then send to plugin",
        p0.buffer()
    );
    let uuid = Uuid::parse_str(PLUGIN_UUID)?;

    match Command::from(cmd_id) {
        Command::Ping => {
            let plugin = LoadablePlugin::new(&uuid);
            let outbuf = plugin.invoke(PluginCommand::Print as u32, PLUGIN_SUBCMD_NULL, &inbuf)?;

            trace_println!(
                "[+] TA received out value {:?} outlen {:?}",
                outbuf,
                outbuf.len()
            );
            trace_println!("[+] TA call invoke_supp_plugin finished");

            Ok(())
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
