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

use optee_teec::{
    Context, Operation, ParamNone, ParamTmpRef, ParamType, ParamValue, Session, Uuid,
};
use proto::{Algo, Command, KeySize, Mode, UUID};

const AES_TEST_BUFFER_SIZE: usize = 4096;
const AES_TEST_KEY_SIZE: usize = 16;
const AES_BLOCK_SIZE: usize = 16;

const DECODE: i8 = 0;
const ENCODE: i8 = 1;

fn prepare_aes(session: &mut Session, encode: i8) -> optee_teec::Result<()> {
    let p2_value = if encode == ENCODE {
        Mode::Encode as u32
    } else {
        Mode::Decode as u32
    };
    let p0 = ParamValue::new(Algo::CTR as u32, 0, ParamType::ValueInput);
    let p1 = ParamValue::new(KeySize::Bit128 as u32, 0, ParamType::ValueInput);
    let p2 = ParamValue::new(p2_value, 0, ParamType::ValueInput);
    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Prepare as u32, &mut operation)?;

    Ok(())
}

fn set_key(session: &mut Session, key: &[u8]) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(key);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    session.invoke_command(Command::SetKey as u32, &mut operation)?;

    Ok(())
}

fn set_iv(session: &mut Session, iv: &[u8]) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(iv);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    session.invoke_command(Command::SetIV as u32, &mut operation)?;

    Ok(())
}

fn cipher_buffer(
    session: &mut Session,
    intext: &[u8],
    outtext: &mut [u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(intext);
    let p1 = ParamTmpRef::new_output(outtext);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);

    session.invoke_command(Command::Cipher as u32, &mut operation)?;

    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    let key = [0xa5u8; AES_TEST_KEY_SIZE];
    let iv = [0x00u8; AES_BLOCK_SIZE];
    let clear = [0x5au8; AES_TEST_BUFFER_SIZE];
    let mut ciph = [0x00u8; AES_TEST_BUFFER_SIZE];
    let mut tmp = [0x00u8; AES_TEST_BUFFER_SIZE];

    println!("Prepare encode operation");
    prepare_aes(&mut session, ENCODE)?;

    println!("Load key in TA");
    set_key(&mut session, &key)?;

    println!("Reset ciphering operation in TA (provides the initial vector)");
    set_iv(&mut session, &iv)?;

    println!("Encode buffer from TA");
    cipher_buffer(&mut session, &clear, &mut ciph)?;

    println!("Prepare decode operation");
    prepare_aes(&mut session, DECODE)?;

    let key = [0xa5u8; AES_TEST_KEY_SIZE];
    println!("Load key in TA");
    set_key(&mut session, &key)?;

    let iv = [0x00u8; AES_BLOCK_SIZE];
    println!("Reset ciphering operation in TA (provides the initial vector)");
    set_iv(&mut session, &iv)?;

    println!("Decode buffer from TA");
    cipher_buffer(&mut session, &ciph, &mut tmp)?;

    if clear.iter().zip(tmp.iter()).all(|(a, b)| a == b) {
        println!("Clear text and decoded text match");
    } else {
        println!("Clear text and decoded text differ => ERROR");
    }
    Ok(())
}
