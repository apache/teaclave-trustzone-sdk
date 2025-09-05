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

use optee_teec::{Context, Operation, ParamNone, ParamTmpRef, Session, Uuid};
use proto::{Command, UUID};

const PUBLIC_KEY_SIZE: usize = 259;
const SIGNATURE_SIZE: usize = 256;

fn sign(
    session: &mut Session,
    message: &[u8],
    public_key: &mut [u8],
    signature: &mut [u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(message);
    let p1 = ParamTmpRef::new_output(public_key);
    let p2 = ParamTmpRef::new_output(signature);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Sign as u32, &mut operation)?;

    Ok(())
}

fn verify(
    session: &mut Session,
    message: &[u8],
    public_key: &[u8],
    signature: &[u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(message);
    let p1 = ParamTmpRef::new_input(public_key);
    let p2 = ParamTmpRef::new_input(signature);

    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

    session.invoke_command(Command::Verify as u32, &mut operation)?;

    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    let message: &[u8] = b"hello,world";
    println!("CA: message: {:?}", &message);
    let mut public_key = [0x00u8; PUBLIC_KEY_SIZE];
    let mut signature = [0x00u8; SIGNATURE_SIZE];

    sign(&mut session, message, &mut public_key, &mut signature)?;
    println!("CA: public key: {:?}", &public_key);
    println!("CA: signature: {:?}", &signature);

    verify(&mut session, message, &public_key, &signature)?;
    println!("Success");

    Ok(())
}
