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

use alloc::boxed::Box;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{AlgorithmId, Asymmetric, OperationMode};
use optee_utee::{ErrorKind, Parameters, Result};
use optee_utee::{GenericObject, TransientObject, TransientObjectType};
use proto::Command;

pub struct RsaCipher {
    pub key: TransientObject,
}

impl Default for RsaCipher {
    fn default() -> Self {
        Self {
            key: TransientObject::null_object(),
        }
    }
}

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters, _sess_ctx: &mut RsaCipher) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session(_sess_ctx: &mut RsaCipher) {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

fn gen_key(rsa: &mut RsaCipher, params: &mut Parameters) -> Result<()> {
    let key_size = unsafe { params.0.as_value()?.a() };
    rsa.key = TransientObject::allocate(TransientObjectType::RsaKeypair, key_size as usize)?;
    rsa.key.generate_key(key_size as usize, &[])?;
    Ok(())
}

fn get_size(rsa: &mut RsaCipher, params: &mut Parameters) -> Result<()> {
    let key_info = rsa.key.info()?;
    unsafe {
        params
            .0
            .as_value()?
            .set_a((key_info.object_size() / 8) as u32)
    };
    Ok(())
}

fn encrypt(rsa: &mut RsaCipher, params: &mut Parameters) -> Result<()> {
    let key_info = rsa.key.info()?;
    let mut p0 = unsafe { params.0.as_memref()? };
    let plain_text = p0.buffer();
    let mut p1 = unsafe { params.1.as_memref()? };
    match Asymmetric::allocate(
        AlgorithmId::RsaesPkcs1V15,
        OperationMode::Encrypt,
        key_info.object_size(),
    ) {
        Err(e) => Err(e),
        Ok(cipher) => {
            cipher.set_key(&rsa.key)?;
            match cipher.encrypt(&[], plain_text) {
                Err(e) => Err(e),
                Ok(cipher_text) => {
                    if cipher_text.len() > p1.buffer().len() {
                        p1.set_updated_size(cipher_text.len());
                        Err(ErrorKind::ShortBuffer.into())
                    } else {
                        p1.buffer().clone_from_slice(&cipher_text);
                        Ok(())
                    }
                }
            }
        }
    }
}

fn decrypt(rsa: &mut RsaCipher, params: &mut Parameters) -> Result<()> {
    let key_info = rsa.key.info()?;
    let mut p0 = unsafe { params.0.as_memref()? };
    let cipher_text = p0.buffer();
    let mut p1 = unsafe { params.1.as_memref()? };
    match Asymmetric::allocate(
        AlgorithmId::RsaesPkcs1V15,
        OperationMode::Decrypt,
        key_info.object_size(),
    ) {
        Err(e) => Err(e),
        Ok(cipher) => {
            cipher.set_key(&rsa.key)?;
            match cipher.decrypt(&[], cipher_text) {
                Err(e) => Err(e),
                Ok(plain_text) => {
                    p1.buffer().clone_from_slice(&plain_text);
                    Ok(())
                }
            }
        }
    }
}

#[ta_invoke_command]
fn invoke_command(sess_ctx: &mut RsaCipher, cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::GenKey => gen_key(sess_ctx, params),
        Command::GetSize => get_size(sess_ctx, params),
        Command::Encrypt => encrypt(sess_ctx, params),
        Command::Decrypt => decrypt(sess_ctx, params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
