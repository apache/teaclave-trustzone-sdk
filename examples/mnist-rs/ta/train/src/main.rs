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

use burn::backend::{ndarray::NdArrayDevice, Autodiff, NdArray};
use optee_utee::prelude::*;
use optee_utee::{ErrorKind, Result};
use proto::train::Command;
use spin::Mutex;

mod trainer;

type NoStdTrainer = trainer::Trainer<Autodiff<NdArray>>;

const DEVICE: NdArrayDevice = NdArrayDevice::Cpu;
static TRAINER: Mutex<Option<NoStdTrainer>> = Mutex::new(Option::None);

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(
    (p0, _, _, _): &mut (
        ParameterMemrefInput<'_>,
        ParameterNone,
        ParameterNone,
        ParameterNone,
    ),
) -> Result<()> {
    let learning_rate = f64::from_le_bytes(p0.get_buffer().try_into().map_err(|err| {
        trace_println!("bad parameter {:?}", err);
        ErrorKind::BadParameters
    })?);
    trace_println!("Initialize with learning_rate: {}", learning_rate);

    let mut trainer = TRAINER.lock();
    trainer.replace(NoStdTrainer::new(DEVICE, learning_rate));

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
fn invoke_command(cmd_id: u32, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    match Command::try_from(cmd_id) {
        Ok(Command::Train) => {
            let images = p0.as_memref_input()?.get_buffer();
            let labels = p1.as_memref_input()?.get_buffer();

            let mut trainer = TRAINER.lock();
            let result = trainer
                .as_mut()
                .ok_or(ErrorKind::CorruptObject)?
                .train(bytemuck::cast_slice(images), labels);
            let bytes = serde_json::to_vec(&result).map_err(|err| {
                trace_println!("unexpected error: {:?}", err);
                ErrorKind::BadState
            })?;
            p2.as_memref_output()?.set_output(bytes)
        }
        Ok(Command::Valid) => {
            let images = p0.as_memref_input()?.get_buffer();
            let labels = p1.as_memref_input()?.get_buffer();

            let trainer = TRAINER.lock();
            let result = trainer
                .as_ref()
                .ok_or(ErrorKind::CorruptObject)?
                .valid(bytemuck::cast_slice(images), labels);

            let bytes = serde_json::to_vec(&result).map_err(|err| {
                trace_println!("unexpected error: {:?}", err);
                ErrorKind::BadState
            })?;
            p2.as_memref_output()?.set_output(bytes)
        }
        Ok(Command::Export) => {
            let trainer = TRAINER.lock();
            let result = trainer
                .as_ref()
                .ok_or(ErrorKind::CorruptObject)?
                .export()
                .map_err(|err| {
                    trace_println!("unexpected error: {:?}", err);
                    ErrorKind::BadState
                })?;
            p0.as_memref_output()?.set_output(result)
        }
        Err(_) => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
