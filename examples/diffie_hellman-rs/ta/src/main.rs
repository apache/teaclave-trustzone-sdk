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
use optee_utee::{AlgorithmId, DeriveKey};
use optee_utee::{
    AttributeId, AttributeMemref, GenericObject, TransientObject, TransientObjectType,
};
use optee_utee::{ErrorKind, Result};
use proto::{Command, KEY_SIZE};

pub struct DiffieHellman {
    pub key: TransientObject,
}

impl Default for DiffieHellman {
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
fn open_session(_params: &mut ParametersNone, _sess_ctx: &mut DiffieHellman) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session(_sess_ctx: &mut DiffieHellman) {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

fn generate_key(dh: &mut DiffieHellman, (p0, p1, p2, p3): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1, p2, p3) = (
        p0.as_memref_input()?,
        p1.as_value_output()?,
        p2.as_memref_output()?,
        p3.as_memref_output()?,
    );
    // Extract prime and base from parameters
    let prime_base_vec = p0.get_buffer();
    let prime_slice = &prime_base_vec[..KEY_SIZE / 8];
    let base_slice = &prime_base_vec[KEY_SIZE / 8..];

    let attr_prime = AttributeMemref::from_ref(AttributeId::DhPrime, prime_slice);
    let attr_base = AttributeMemref::from_ref(AttributeId::DhBase, base_slice);

    // Generate key pair
    dh.key = TransientObject::allocate(TransientObjectType::DhKeypair, KEY_SIZE)?;

    dh.key
        .generate_key(KEY_SIZE, &[attr_prime.into(), attr_base.into()])?;
    {
        let key_size = dh
            .key
            .ref_attribute(AttributeId::DhPublicValue, p2.get_buffer_mut())?;
        p2.set_updated_size(key_size)?;
        p1.set_a(key_size as u32);
    }

    {
        let key_size = dh
            .key
            .ref_attribute(AttributeId::DhPrivateValue, p3.get_buffer_mut())?;
        p3.set_updated_size(key_size)?;
        p1.set_b(key_size as u32);
    }
    Ok(())
}

fn derive_key(dh: &mut DiffieHellman, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1, p2) = (
        p0.as_memref_input()?,
        p1.as_memref_output()?,
        p2.as_value_output()?,
    );
    let received_public = AttributeMemref::from_ref(AttributeId::DhPublicValue, p0.get_buffer());

    let mut operation = DeriveKey::allocate(AlgorithmId::DhDeriveSharedSecret, KEY_SIZE)?;
    operation.set_key(&dh.key)?;
    let mut derived_key = TransientObject::allocate(TransientObjectType::GenericSecret, KEY_SIZE)?;
    operation.derive(&[received_public.into()], &mut derived_key);
    let key_size = derived_key.ref_attribute(AttributeId::SecretValue, p1.get_buffer_mut())?;
    p1.set_updated_size(key_size)?;
    p2.set_a(key_size as u32);
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(
    sess_ctx: &mut DiffieHellman,
    cmd_id: u32,
    params: &mut ParametersAny<'_>,
) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::GenerateKey => generate_key(sess_ctx, params),
        Command::DeriveKey => derive_key(sess_ctx, params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
