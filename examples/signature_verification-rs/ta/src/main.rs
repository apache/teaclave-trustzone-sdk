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

use alloc::vec;
use optee_utee::prelude::*;
use optee_utee::{AlgorithmId, Asymmetric, AttributeId, AttributeMemref, Digest, OperationMode};
use optee_utee::{ErrorKind, Result};
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

fn sign((p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let p0 = p0.as_memref_input()?;
    let p1 = p1.as_memref_output()?;
    let p2 = p2.as_memref_output()?;
    let message = p0.get_buffer();
    trace_println!("[+] message: {:?}", message);

    let rsa_key = TransientObject::allocate(TransientObjectType::RsaKeypair, 2048_usize)?;

    rsa_key.generate_key(2048_usize, &[])?;

    {
        let buffer = p1.get_buffer_mut();
        let modulus_len = rsa_key.ref_attribute(AttributeId::RsaModulus, buffer)?;
        let exp_len =
            rsa_key.ref_attribute(AttributeId::RsaPublicExponent, &mut buffer[modulus_len..])?;
        p1.set_updated_size(modulus_len + exp_len)?;
    };

    let mut hash = [0u8; 32];
    let dig = Digest::allocate(AlgorithmId::Sha256)?;

    dig.do_final(message, &mut hash)?;

    let key_info = rsa_key.info()?;

    let mut rsa = Asymmetric::allocate(
        AlgorithmId::RsassaPkcs1V15Sha256,
        OperationMode::Sign,
        key_info.object_size(),
    )?;

    rsa.set_key(&rsa_key)?;
    let len = rsa.sign_digest(&[], &hash, p2.get_buffer_mut())?;
    p2.set_updated_size(len)?;
    Ok(())
}

fn verify((p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let p0 = p0.as_memref_input()?;
    let p1 = p1.as_memref_input()?;
    let p2 = p2.as_memref_input()?;

    let message = p0.get_buffer();
    let mut pub_key_mod = vec![0u8; 256];
    let mut pub_key_exp = vec![0u8; p1.get_buffer().len() - 256];
    let signature = p2.get_buffer();

    pub_key_mod.copy_from_slice(&p1.get_buffer()[..256]);
    pub_key_exp.copy_from_slice(&p1.get_buffer()[256..]);

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

    let mut rsa = Asymmetric::allocate(
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
fn invoke_command(cmd_id: u32, params: &mut ParametersAny<'_>) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Sign => sign(params),
        Command::Verify => verify(params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
