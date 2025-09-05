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

use optee_teec::{Context, ErrorKind, Operation, ParamNone, ParamTmpRef, Session, Uuid};
use proto::{Command, UUID};
use std::ffi::CString;

const TEST_OBJECT_SIZE: usize = 7000;

fn read_secure_object(
    session: &mut Session,
    obj_id: &[u8],
    obj_data: &mut [u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(obj_id);
    let p1 = ParamTmpRef::new_output(obj_data);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);

    session.invoke_command(Command::Read as u32, &mut operation)?;

    println!("- Read back the object");
    Ok(())
}

fn write_secure_object(
    session: &mut Session,
    obj_id: &[u8],
    obj_data: &[u8],
) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(obj_id);
    let p1 = ParamTmpRef::new_input(obj_data);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);

    session.invoke_command(Command::Write as u32, &mut operation)?;

    println!("- Create and load object in the TA secure storage");
    Ok(())
}

fn delete_secure_object(session: &mut Session, obj_id: &[u8]) -> optee_teec::Result<()> {
    let p0 = ParamTmpRef::new_input(obj_id);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    session.invoke_command(Command::Delete as u32, &mut operation)?;

    println!("- Delete the object");
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    let obj1_id = CString::new("object#1")
        .map_err(|e| {
            eprintln!("Failed to create CString for object#1: {}", e);
            ErrorKind::BadParameters
        })?
        .into_bytes_with_nul();
    let obj1_data = [0xA1u8; TEST_OBJECT_SIZE];
    let mut read_data = [0x00u8; TEST_OBJECT_SIZE];

    println!("\nTest on object \"object#1\"");
    write_secure_object(&mut session, obj1_id.as_slice(), &obj1_data)?;
    read_secure_object(&mut session, obj1_id.as_slice(), &mut read_data)?;

    if obj1_data.iter().zip(read_data.iter()).all(|(a, b)| a == b) {
        println!("- Content read-out correctly");
    } else {
        println!("- Unexpected content found in secure storage");
    }
    delete_secure_object(&mut session, &obj1_id)?;

    let obj2_id = CString::new("object#2")
        .map_err(|e| {
            eprintln!("Failed to create CString for object#2: {}", e);
            ErrorKind::BadParameters
        })?
        .into_bytes_with_nul();

    println!("\nTest on object \"object#2\"");
    match read_secure_object(&mut session, obj2_id.as_slice(), &mut read_data) {
        Err(e) => {
            if e.kind() != ErrorKind::ItemNotFound {
                println!("{}", e);
                return Err(e);
            } else {
                println!("- Object not found in TA secure storage, create it");
                let obj2_data = [0xB1u8; TEST_OBJECT_SIZE];
                write_secure_object(&mut session, &obj2_id, &obj2_data)?;
            }
        }

        Ok(()) => {
            println!("- Object found in TA secure storage, delete it");
            delete_secure_object(&mut session, &obj2_id)?;
        }
    }

    println!("\nWe're done, close and release TEE resources");
    Ok(())
}
