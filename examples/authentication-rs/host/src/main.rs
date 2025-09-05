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
use proto::{Command, Mode, AAD_LEN, BUFFER_SIZE, KEY_SIZE, TAG_LEN, UUID};

fn prepare(
    session: &mut Session,
    mode: Mode,
    nonce: &[u8],
    key: &[u8],
    aad: &[u8],
) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(mode as u32, 0, ParamType::ValueInput);
    let p1 = ParamTmpRef::new_input(nonce);
    let p2 = ParamTmpRef::new_input(key);
    let p3 = ParamTmpRef::new_input(aad);
    let mut operation = Operation::new(0, p0, p1, p2, p3);

    session.invoke_command(Command::Prepare as u32, &mut operation)?;
    Ok(())
}

fn update(session: &mut Session, src: &[u8], res: &mut [u8]) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(src);
    let p1 = ParamTmpRef::new_output(res);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);

    session.invoke_command(Command::Update as u32, &mut operation)?;

    Ok(())
}

fn encrypt_final(
    session: &mut Session,
    src: &[u8],
    res: &mut [u8],
    tag: &mut [u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(src);
    let p1 = ParamTmpRef::new_output(res);
    let p2 = ParamTmpRef::new_output(tag);
    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::EncFinal as u32, &mut operation)?;
    Ok(())
}

fn decrypt_final(
    session: &mut Session,
    src: &[u8],
    res: &mut [u8],
    tag: &[u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(src);
    let p1 = ParamTmpRef::new_output(res);
    let p2 = ParamTmpRef::new_input(tag);
    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::DecFinal as u32, &mut operation)?;
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    let key = [0xa5u8; KEY_SIZE];
    let nonce = [0x00u8; 8];
    let aad = [0xffu8; AAD_LEN];
    let clear1 = [0x5au8; BUFFER_SIZE + 3];
    let clear2 = [0xa5u8; BUFFER_SIZE - 3];
    let mut ciph1 = [0x00u8; BUFFER_SIZE];
    let mut ciph2 = [0x00u8; BUFFER_SIZE];
    let mut tmp1 = [0x00u8; BUFFER_SIZE];
    let mut tmp2 = [0x00u8; BUFFER_SIZE];
    let mut tag = [0x00u8; TAG_LEN];

    prepare(&mut session, Mode::Encrypt, &nonce, &key, &aad)?;
    update(&mut session, &clear1, &mut ciph1)?;
    encrypt_final(&mut session, &clear2, &mut ciph2, &mut tag)?;

    prepare(&mut session, Mode::Decrypt, &nonce, &key, &aad)?;
    update(&mut session, &ciph1, &mut tmp1)?;
    decrypt_final(&mut session, &ciph2, &mut tmp2, &tag)?;

    let mut clear_total = clear1.to_vec();
    clear_total.extend_from_slice(&clear2);
    let mut tmp_total = tmp1.to_vec();
    tmp_total.extend_from_slice(&tmp2);
    if clear_total
        .iter()
        .zip(tmp_total.iter())
        .all(|(a, b)| a == b)
    {
        println!("Clear text and decoded text match");
    } else {
        println!("Clear text and decoded text differ => ERROR");
    }

    println!("Success");
    Ok(())
}
