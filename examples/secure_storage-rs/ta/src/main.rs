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

use optee_utee::prelude::*;
use optee_utee::{DataFlag, GenericObject, ObjectStorageConstants, PersistentObject};
use optee_utee::{ErrorKind, Result};
use proto::Command;

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
fn invoke_command(cmd_id: u32, params: &mut ParametersAny<'_>) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Write => create_raw_object(params),
        Command::Read => read_raw_object(params),
        Command::Delete => delete_object(params),
        _ => Err(ErrorKind::NotSupported.into()),
    }
}

pub fn delete_object((p0, _, _, _): &mut ParametersAny<'_>) -> Result<()> {
    // use to_vec to copy into tee memory
    let obj_id = p0.as_memref_input()?.get_buffer().to_vec();

    match PersistentObject::open(
        ObjectStorageConstants::Private,
        &obj_id,
        DataFlag::ACCESS_READ | DataFlag::ACCESS_WRITE_META,
    ) {
        Err(e) => Err(e),

        Ok(object) => {
            object.close_and_delete()?;
            Ok(())
        }
    }
}

pub fn create_raw_object((p0, p1, _, _): &mut ParametersAny<'_>) -> Result<()> {
    // use to_vec to copy into tee memory
    let obj_id = p0.as_memref_input()?.get_buffer().to_vec();
    let data_buffer = p1.as_memref_input()?.get_buffer().to_vec();

    let obj_data_flag = DataFlag::ACCESS_READ
        | DataFlag::ACCESS_WRITE
        | DataFlag::ACCESS_WRITE_META
        | DataFlag::OVERWRITE;

    let init_data: [u8; 0] = [0; 0];

    let mut object = PersistentObject::create(
        ObjectStorageConstants::Private,
        &obj_id,
        obj_data_flag,
        None,
        &init_data,
    )?;
    match object.write(&data_buffer) {
        Ok(()) => Ok(()),
        Err(e_write) => {
            object.close_and_delete()?;
            Err(e_write)
        }
    }
}

pub fn read_raw_object((p0, p1, _, _): &mut ParametersAny<'_>) -> Result<()> {
    // use to_vec to copy into tee memory
    let obj_id = p0.as_memref_input()?.get_buffer().to_vec();
    let p1 = p1.as_memref_output()?;

    let mut object = PersistentObject::open(
        ObjectStorageConstants::Private,
        &obj_id,
        DataFlag::ACCESS_READ | DataFlag::SHARE_READ,
    )?;
    let obj_info = object.info()?;

    let read_bytes = object.read(p1.get_buffer_mut())?;
    if read_bytes != obj_info.data_size() as u32 {
        return Err(ErrorKind::ExcessData.into());
    }

    p1.set_updated_size(read_bytes as usize)?;

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
