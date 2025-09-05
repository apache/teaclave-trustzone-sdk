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

use optee_utee::BigInt;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;

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

fn compare(n0: &BigInt, n1: &BigInt) -> Result<()> {
    match n0.compare_big_int(n1) {
        0 => trace_println!("{} == {}.", n0, n1),
        res if res > 0 => trace_println!("{} > {}.", n0, n1),
        _ => trace_println!("{} < {}.", n0, n1),
    }
    Ok(())
}

fn convert(n0: &BigInt, n1: &BigInt) -> Result<()> {
    trace_println!(
        "{} in u8 array is {:x?}.",
        n0,
        n0.convert_to_octet_string()?
    );
    trace_println!("{} in i32 is {}.", n1, n1.convert_to_s32()?);
    Ok(())
}

fn add(n0: &BigInt, n1: &BigInt) -> Result<()> {
    let res = BigInt::add(n0, n1);
    trace_println!("{} + {} = {}.", n0, n1, res);
    Ok(())
}

fn sub(n0: &BigInt, n1: &BigInt) -> Result<()> {
    let res = BigInt::sub(n0, n1);
    trace_println!("{} - {} = {}.", n0, n1, res);
    Ok(())
}

fn multiply(n0: &BigInt, n1: &BigInt) -> Result<()> {
    let res = BigInt::multiply(n0, n1);
    trace_println!("{} * {} = {}.", n0, n1, res);
    Ok(())
}

fn divide(n0: &BigInt, n1: &BigInt) -> Result<()> {
    let (quot, rem) = BigInt::divide(n0, n1);
    trace_println!("{} / {} = {}, ramians {}.", n0, n1, quot, rem);
    Ok(())
}

fn module(n0: &BigInt, n1: &BigInt) -> Result<()> {
    let res = BigInt::module(n0, n1);
    trace_println!("{} % {} = {}.", n0, n1, res);
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let mut n0_buffer = unsafe { params.0.as_memref()? };
    let n1_value = unsafe { params.1.as_value()? };

    let mut n0 = BigInt::new(64);
    let mut n1 = BigInt::new(2);

    n0.convert_from_octet_string(n0_buffer.buffer(), 0)?;
    n1.convert_from_s32(n1_value.a() as i32);

    match Command::from(cmd_id) {
        Command::Compare => compare(&n0, &n1),
        Command::Convert => convert(&n0, &n1),
        Command::Add => add(&n0, &n1),
        Command::Sub => sub(&n0, &n1),
        Command::Multiply => multiply(&n0, &n1),
        Command::Divide => divide(&n0, &n1),
        Command::Module => module(&n0, &n1),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
