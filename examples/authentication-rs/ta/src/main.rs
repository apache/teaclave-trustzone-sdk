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

use alloc::boxed::Box;
use alloc::vec;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{AlgorithmId, OperationMode, AE};
use optee_utee::{AttributeId, AttributeMemref, TransientObject, TransientObjectType};
use optee_utee::{ErrorKind, Parameters, Result};
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
fn open_session(_params: &mut Parameters, _sess_ctx: &mut AEOp) -> Result<()> {
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
fn invoke_command(sess_ctx: &mut AEOp, cmd_id: u32, params: &mut Parameters) -> Result<()> {
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

pub fn prepare(ae: &mut AEOp, params: &mut Parameters) -> Result<()> {
    let p0 = unsafe { params.0.as_value()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_memref()? };
    let mut p3 = unsafe { params.3.as_memref()? };
    let mode = match Mode::from(p0.a()) {
        Mode::Encrypt => OperationMode::Encrypt,
        Mode::Decrypt => OperationMode::Decrypt,
        _ => OperationMode::IllegalValue,
    };
    let nonce = p1.buffer();
    let key = p2.buffer();
    let aad = p3.buffer();

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

pub fn update(digest: &mut AEOp, params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let src = p0.buffer();
    let res = p1.buffer();
    digest.op.update(src, res)?;
    Ok(())
}

pub fn encrypt_final(digest: &mut AEOp, params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_memref()? };

    let mut clear = vec![0; p0.buffer().len()];
    clear.copy_from_slice(p0.buffer());
    let mut ciph = vec![0; p1.buffer().len()];
    ciph.copy_from_slice(p1.buffer());
    let mut tag = vec![0; p2.buffer().len()];
    tag.copy_from_slice(p2.buffer());

    match digest.op.encrypt_final(&clear, &mut ciph, &mut tag) {
        Err(e) => Err(e),
        Ok((_ciph_len, _tag_len)) => {
            p0.buffer().copy_from_slice(&clear);
            p1.buffer().copy_from_slice(&ciph);
            p2.buffer().copy_from_slice(&tag);

            Ok(())
        }
    }
}

pub fn decrypt_final(digest: &mut AEOp, params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_memref()? };

    let mut clear = vec![0; p0.buffer().len()];
    clear.copy_from_slice(p0.buffer());
    let mut ciph = vec![0; p1.buffer().len()];
    ciph.copy_from_slice(p1.buffer());
    let mut tag = vec![0; p2.buffer().len()];
    tag.copy_from_slice(p2.buffer());

    match digest.op.decrypt_final(&clear, &mut ciph, &tag) {
        Err(e) => Err(e),
        Ok(_clear_len) => {
            p0.buffer().copy_from_slice(&clear);
            p1.buffer().copy_from_slice(&ciph);
            p2.buffer().copy_from_slice(&tag);

            Ok(())
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
