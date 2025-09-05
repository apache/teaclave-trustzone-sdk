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

use alloc::vec;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{DataFlag, GenericObject, ObjectStorageConstants, PersistentObject};
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
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Write => create_raw_object(params),
        Command::Read => read_raw_object(params),
        Command::Delete => delete_object(params),
        _ => Err(ErrorKind::NotSupported.into()),
    }
}

pub fn delete_object(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };

    let mut obj_id = vec![0; p0.buffer().len()];
    obj_id.copy_from_slice(p0.buffer());

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

pub fn create_raw_object(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };

    let mut obj_id = vec![0; p0.buffer().len()];
    obj_id.copy_from_slice(p0.buffer());
    let mut data_buffer = vec![0; p1.buffer().len()];
    data_buffer.copy_from_slice(p1.buffer());

    let obj_data_flag = DataFlag::ACCESS_READ
        | DataFlag::ACCESS_WRITE
        | DataFlag::ACCESS_WRITE_META
        | DataFlag::OVERWRITE;

    let init_data: [u8; 0] = [0; 0];

    match PersistentObject::create(
        ObjectStorageConstants::Private,
        &obj_id,
        obj_data_flag,
        None,
        &init_data,
    ) {
        Err(e) => Err(e),

        Ok(mut object) => match object.write(&data_buffer) {
            Ok(()) => Ok(()),
            Err(e_write) => {
                object.close_and_delete()?;
                Err(e_write)
            }
        },
    }
}

pub fn read_raw_object(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut obj_id = vec![0; p0.buffer().len()];
    obj_id.copy_from_slice(p0.buffer());

    let mut data_buffer = vec![0; p1.buffer().len()];
    data_buffer.copy_from_slice(p1.buffer());

    match PersistentObject::open(
        ObjectStorageConstants::Private,
        &obj_id,
        DataFlag::ACCESS_READ | DataFlag::SHARE_READ,
    ) {
        Err(e) => Err(e),

        Ok(object) => {
            let obj_info = object.info()?;

            if obj_info.data_size() > p1.buffer().len() {
                p1.set_updated_size(obj_info.data_size());
                return Err(ErrorKind::ShortBuffer.into());
            }
            let read_bytes = object.read(&mut data_buffer)?;
            if read_bytes != obj_info.data_size() as u32 {
                return Err(ErrorKind::ExcessData.into());
            }

            p1.set_updated_size(read_bytes as usize);
            p1.buffer().copy_from_slice(&data_buffer);

            Ok(())
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
