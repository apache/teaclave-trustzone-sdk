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
use optee_utee::{AlgorithmId, Digest};
use optee_utee::{ErrorKind, Result};
use proto::Command;

pub struct DigestOp {
    pub op: Digest,
}

impl Default for DigestOp {
    // This is related to our TA session context design, which requires the struct to implement
    // the Default trait. Revising this design should be future work, so temporary allow the unwrap() usage.
    #[allow(clippy::unwrap_used)]
    fn default() -> Self {
        Self {
            op: Digest::allocate(AlgorithmId::Sha256).unwrap(),
        }
    }
}

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut ParametersNone, _sess_ctx: &mut DigestOp) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session(_sess_ctx: &mut DigestOp) {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(
    sess_ctx: &mut DigestOp,
    cmd_id: u32,
    params: &mut ParametersAny<'_>,
) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Update => update(sess_ctx, params),
        Command::DoFinal => do_final(sess_ctx, params),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

pub fn update(digest: &mut DigestOp, (p0, _, _, _): &mut ParametersAny<'_>) -> Result<()> {
    let buffer = p0.as_memref_input()?.get_buffer();
    digest.op.update(buffer);
    Ok(())
}

pub fn do_final(digest: &mut DigestOp, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    let (p0, p1, p2) = (
        p0.as_memref_input()?,
        p1.as_memref_output()?,
        p2.as_value_output()?,
    );
    let input = p0.get_buffer();
    let length = digest.op.do_final(input, p1.get_buffer_mut())?;
    p2.set_a(length as u32);
    p1.set_updated_size(length)?;
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
