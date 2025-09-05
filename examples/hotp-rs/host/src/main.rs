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
    Context, ErrorKind, Operation, ParamNone, ParamTmpRef, ParamType, ParamValue, Session, Uuid,
};
use proto::{Command, UUID};

const TEST_SIZE: usize = 10;
const SIZE_K: usize = 20;
const RFC4226_TEST_VALUES: [u32; TEST_SIZE] = [
    755224, 287082, 359152, 969429, 338314, 254676, 287922, 162583, 399871, 520489,
];

fn register_shared_key(session: &mut Session) -> optee_teec::Result<()> {
    let k: [u8; SIZE_K] = [
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35,
        0x36, 0x37, 0x38, 0x39, 0x30,
    ];

    let p0 = ParamTmpRef::new_input(&k);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    session.invoke_command(Command::RegisterSharedKey as u32, &mut operation)?;
    Ok(())
}

fn get_hotp(session: &mut Session) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(0, 0, ParamType::ValueOutput);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    for &expected_value in &RFC4226_TEST_VALUES {
        session.invoke_command(Command::GetHOTP as u32, &mut operation)?;
        let (p0, _, _, _) = operation.parameters();
        let hotp_value = p0.a();

        println!("Get HOTP: {}", hotp_value);

        if hotp_value != expected_value {
            println!("Wrong value get! Expected value: {}", expected_value);
            return Err(ErrorKind::Generic.into());
        }
    }
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    register_shared_key(&mut session)?;
    get_hotp(&mut session)?;

    println!("Success");
    Ok(())
}
