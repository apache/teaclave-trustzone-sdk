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
use optee_utee::{AlgorithmId, Asymmetric, AttributeId, AttributeMemref, Digest, OperationMode};
use optee_utee::{ErrorKind, Parameters, Result};
use optee_utee::{GenericObject, TransientObject, TransientObjectType};
use proto::Command;

pub struct RsaSign {
    pub key: TransientObject,
}

impl Default for RsaSign {
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

fn sign(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_memref()? };
    let message = p0.buffer();
    let mut pub_key_size: usize = 0;
    trace_println!("[+] message: {:?}", &message);

    let rsa_key = TransientObject::allocate(TransientObjectType::RsaKeypair, 2048_usize)?;

    rsa_key.generate_key(2048_usize, &[])?;

    match rsa_key.ref_attribute(AttributeId::RsaModulus, p1.buffer()) {
        Ok(len) => {
            pub_key_size += len;
            Ok(())
        }
        Err(e) => Err(e),
    }?;

    match rsa_key.ref_attribute(
        AttributeId::RsaPublicExponent,
        &mut p1.buffer()[pub_key_size..],
    ) {
        Ok(len) => {
            pub_key_size += len;
            Ok(())
        }
        Err(e) => Err(e),
    }?;

    p1.set_updated_size(pub_key_size);

    let mut hash = [0u8; 32];
    let dig = Digest::allocate(AlgorithmId::Sha256)?;

    dig.do_final(message, &mut hash)?;

    let key_info = rsa_key.info()?;
    let signature = p2.buffer();

    let rsa = Asymmetric::allocate(
        AlgorithmId::RsassaPkcs1V15Sha256,
        OperationMode::Sign,
        key_info.object_size(),
    )?;

    rsa.set_key(&rsa_key)?;
    match rsa.sign_digest(&[], &hash, signature) {
        Ok(len) => {
            trace_println!("[+] signature: {:?}", p2.buffer());
            p2.set_updated_size(len);
            Ok(())
        }
        Err(e) => {
            trace_println!("[+] error: {:?}", e);
            Err(ErrorKind::SignatureInvalid.into())
        }
    }
}

fn verify(params: &mut Parameters) -> Result<()> {
    let mut p0 = unsafe { params.0.as_memref()? };
    let mut p1 = unsafe { params.1.as_memref()? };
    let mut p2 = unsafe { params.2.as_memref()? };

    let message = p0.buffer();
    let mut pub_key_mod = vec![0u8; 256];
    let mut pub_key_exp = vec![0u8; p1.buffer().len() - 256];
    let signature = p2.buffer();

    pub_key_mod.copy_from_slice(&p1.buffer()[..256]);
    pub_key_exp.copy_from_slice(&p1.buffer()[256..]);

    trace_println!("[+] message: {:?}", &message);
    trace_println!("[+] public_key_mod: {:?}", &pub_key_mod);
    trace_println!("[+] public_key_exp: {:?}", &pub_key_exp);
    trace_println!("[+] signature: {:?}", &signature);

    let mut rsa_pub_key = TransientObject::allocate(TransientObjectType::RsaPublicKey, 2048_usize)?;

    let mod_attr = AttributeMemref::from_ref(AttributeId::RsaModulus, &pub_key_mod);
    let exp_attr = AttributeMemref::from_ref(AttributeId::RsaPublicExponent, &pub_key_exp);

    rsa_pub_key.populate(&[mod_attr.into(), exp_attr.into()])?;

    let mut hash = [0u8; 32];
    let dig = Digest::allocate(AlgorithmId::Sha256)?;

    dig.do_final(message, &mut hash)?;

    let key_info = rsa_pub_key.info()?;

    let rsa = Asymmetric::allocate(
        AlgorithmId::RsassaPkcs1V15Sha256,
        OperationMode::Verify,
        key_info.object_size(),
    )?;

    rsa.set_key(&rsa_pub_key)?;
    match rsa.verify_digest(&[], &hash, signature) {
        Ok(_) => {
            trace_println!("[+] verify ok");
            Ok(())
        }
        Err(e) => {
            trace_println!("[+] error: {:?}", e);
            Err(ErrorKind::SignatureInvalid.into())
        }
    }
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Sign => sign(params),
        Command::Verify => verify(params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
