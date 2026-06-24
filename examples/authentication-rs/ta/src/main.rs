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
use optee_utee::{AlgorithmId, OperationMode, AE};
use optee_utee::{AttributeId, AttributeMemref, TransientObject, TransientObjectType};
use optee_utee::{ErrorKind, Result};
use proto::{Command, Mode, AAD_LEN, BUFFER_SIZE, KEY_SIZE, TAG_LEN};

pub const PAYLOAD_NUMBER: usize = 2;

pub struct AEOp {
    pub op: AE,
}

impl Default for AEOp {
    fn default() -> Self {
        Self { op: AE::null() }
    }
}

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut ParametersNone, _sess_ctx: &mut AEOp) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session(_sess_ctx: &mut AEOp) {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(sess_ctx: &mut AEOp, cmd_id: u32, params: &mut ParametersAny<'_>) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Prepare => {
            trace_println!("[+] TA prepare");
            prepare(sess_ctx, params)
        }
        Command::Update => {
            trace_println!("[+] TA update");
            update(sess_ctx, params)
        }
        Command::EncFinal => {
            trace_println!("[+] TA encrypt_final");
            encrypt_final(sess_ctx, params)
        }
        Command::DecFinal => {
            trace_println!("[+] TA decrypt_final");
            decrypt_final(sess_ctx, params)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

pub fn prepare(ae: &mut AEOp, (p0, p1, p2, p3): &mut ParametersAny<'_>) -> Result<()> {
    let mode = match Mode::from(p0.as_value_input()?.get_a()) {
        Mode::Encrypt => OperationMode::Encrypt,
        Mode::Decrypt => OperationMode::Decrypt,
        _ => OperationMode::IllegalValue,
    };
    let nonce = p1.as_memref_input()?.get_buffer();
    let key = p2.as_memref_input()?.get_buffer();
    let aad = p3.as_memref_input()?.get_buffer();

    ae.op = AE::allocate(AlgorithmId::AesCcm, mode, KEY_SIZE * 8)?;

    let mut key_object = TransientObject::allocate(TransientObjectType::Aes, KEY_SIZE * 8)?;
    let attr = AttributeMemref::from_ref(AttributeId::SecretValue, key);
    key_object.populate(&[attr.into()])?;
    ae.op.set_key(&key_object)?;
    ae.op
        .init(nonce, TAG_LEN * 8, AAD_LEN, BUFFER_SIZE * PAYLOAD_NUMBER)?;
    ae.op.update_aad(aad);
    Ok(())
}

pub fn update(digest: &mut AEOp, (p0, p1, _, _): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1) = (p0.as_memref_input()?, p1.as_memref_output()?);
    let size = digest.op.update(p0.get_buffer(), p1.get_buffer_mut())?;
    p1.set_updated_size(size)?;
    Ok(())
}

pub fn encrypt_final(digest: &mut AEOp, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1, p2) = (
        p0.as_memref_input()?,
        p1.as_memref_output()?,
        p2.as_memref_output()?,
    );

    let (ciph_len, tag_len) =
        digest
            .op
            .encrypt_final(p0.get_buffer(), p1.get_buffer_mut(), p2.get_buffer_mut())?;
    p1.set_updated_size(ciph_len)?;
    p2.set_updated_size(tag_len)?;
    Ok(())
}

pub fn decrypt_final(digest: &mut AEOp, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1, p2) = (
        p0.as_memref_input()?,
        p1.as_memref_output()?,
        p2.as_memref_input()?,
    );

    let len = digest
        .op
        .decrypt_final(p0.get_buffer(), p1.get_buffer_mut(), p2.get_buffer())?;
    p1.set_updated_size(len)?;
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
