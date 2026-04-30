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

use optee_teec::{
    macros::{plugin_init, plugin_invoke},
    ErrorKind, Result,
};
use proto::PluginCommand;

#[plugin_init]
fn init() -> Result<()> {
    println!("*plugin*: init, version: {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[plugin_invoke]
fn invoke(params: &mut optee_teec::PluginParameters) -> optee_teec::Result<()> {
    println!("*plugin*: invoke");
    match PluginCommand::from(params.cmd) {
        PluginCommand::Print => {
            let input = params.get_buffer();
            println!(
                "*plugin*: receive value: {:?} length {:?}",
                input,
                input.len()
            );

            let send_slice: [u8; 9] = [0x40; 9];
            params.set_buf_from_slice(&send_slice)?;
            println!(
                "*plugin*: send value: {:?} length {:?} to ta",
                send_slice,
                send_slice.len()
            );
            Ok(())
        }
        _ => {
            println!("Unsupported plugin command: {:?}", params.cmd);
            Err(ErrorKind::BadParameters.into())
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/plugin_static.rs"));
