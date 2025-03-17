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

use burn::{
    backend::{ndarray::NdArrayDevice, NdArray},
    tensor::cast::ToElement,
};

use common::{copy_to_output, Model};
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Image;
use spin::Mutex;

type NoStdModel = Model<NdArray>;
const DEVICE: NdArrayDevice = NdArrayDevice::Cpu;
static MODEL: Mutex<Option<NoStdModel>> = Mutex::new(Option::None);

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };

    let mut model = MODEL.lock();
    model.replace(Model::import(&DEVICE, p0.buffer().to_vec()).map_err(|err| {
        trace_println!("import failed: {:?}", err);
        ErrorKind::BadParameters
    })?);

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
fn invoke_command(_cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let mut p0 = unsafe { params.0.as_memref()? };
    let images: &[Image] = bytemuck::cast_slice(p0.buffer());
    let input = NoStdModel::images_to_tensors(&DEVICE, images);

    let output = MODEL
        .lock()
        .as_ref()
        .ok_or(ErrorKind::CorruptObject)?
        .forward(input);
    let result: alloc::vec::Vec<u8> = output
        .iter_dim(0)
        .map(|v| {
            let data = burn::tensor::activation::softmax(v, 1);
            data.argmax(1).into_scalar().to_u8()
        })
        .collect();

    copy_to_output(&mut params.1, &result)
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
