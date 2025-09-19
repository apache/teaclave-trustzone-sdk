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
use optee_utee::{AlgorithmId, Mac};
use optee_utee::{AttributeId, AttributeMemref, TransientObject, TransientObjectType};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;

pub const SHA1_HASH_SIZE: usize = 20;
pub const MAX_KEY_SIZE: usize = 64;
pub const MIN_KEY_SIZE: usize = 10;
pub const DBC2_MODULO: u32 = 1000000;

pub struct HmacOtp {
    pub counter: [u8; 8],
    pub key: [u8; MAX_KEY_SIZE],
    pub key_len: usize,
}

impl Default for HmacOtp {
    fn default() -> Self {
        Self {
            counter: [0u8; 8],
            key: [0u8; MAX_KEY_SIZE],
            key_len: 0,
        }
    }
}

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters, _sess_ctx: &mut HmacOtp) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session(_sess_ctx: &mut HmacOtp) {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(sess_ctx: &mut HmacOtp, cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::RegisterSharedKey => register_shared_key(sess_ctx, params),
        Command::GetHOTP => get_hotp(sess_ctx, params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

pub fn register_shared_key(hotp: &mut HmacOtp, params: &mut Parameters) -> Result<()> {
    let mut p = unsafe { params.0.as_memref()? };
    let buffer = p.buffer();
    hotp.key_len = buffer.len();
    hotp.key[..hotp.key_len].clone_from_slice(buffer);
    Ok(())
}

pub fn get_hotp(hotp: &mut HmacOtp, params: &mut Parameters) -> Result<()> {
    let mut mac: [u8; SHA1_HASH_SIZE] = [0x0; SHA1_HASH_SIZE];

    hmac_sha1(hotp, &mut mac)?;

    for i in (0..hotp.counter.len()).rev() {
        hotp.counter[i] += 1;
        if hotp.counter[i] > 0 {
            break;
        }
    }
    let hotp_val = truncate(&mut mac);
    let mut p = unsafe { params.0.as_value()? };
    p.set_a(hotp_val);
    Ok(())
}

pub fn hmac_sha1(hotp: &mut HmacOtp, out: &mut [u8]) -> Result<usize> {
    if hotp.key_len < MIN_KEY_SIZE || hotp.key_len > MAX_KEY_SIZE {
        return Err(ErrorKind::BadParameters.into());
    }

    let mac = Mac::allocate(AlgorithmId::HmacSha1, hotp.key_len * 8)?;
    let mut key_object =
        TransientObject::allocate(TransientObjectType::HmacSha1, hotp.key_len * 8)?;
    //KEY size can be larger than hotp.key_len
    let mut tmp_key = hotp.key.to_vec();
    tmp_key.truncate(hotp.key_len);
    let attr = AttributeMemref::from_ref(AttributeId::SecretValue, &tmp_key);
    key_object.populate(&[attr.into()])?;
    mac.set_key(&key_object)?;

    let iv = [0u8; 0];
    mac.init(&iv);
    mac.update(&hotp.counter);
    let message = [0u8; 0];
    let out_len = mac.compute_final(&message, out)?;
    Ok(out_len)
}

pub fn truncate(hmac_result: &mut [u8]) -> u32 {
    let mut bin_code: u32;
    let offset: usize = (hmac_result[19] & 0xf) as usize;

    bin_code = ((hmac_result[offset] & 0x7f) as u32) << 24
        | (hmac_result[offset + 1] as u32) << 16
        | (hmac_result[offset + 2] as u32) << 8
        | (hmac_result[offset + 3] as u32);

    bin_code %= DBC2_MODULO;
    bin_code
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
