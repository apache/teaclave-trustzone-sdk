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
use proto::{Command, Point, UUID};

fn serde(session: &mut Session) -> optee_teec::Result<()> {
    let mut buffer = [0u8; 128];
    let p0 = ParamTmpRef::new_output(&mut buffer);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    session.invoke_command(Command::DefaultOp as u32, &mut operation)?;
    let updated_size = operation.parameters().0.updated_size();

    let p: Point = serde_json::from_slice(&buffer[..updated_size]).map_err(|e| {
        eprintln!("Failed to deserialize point: {}", e);
        ErrorKind::BadParameters
    })?;
    println!("{:?}", p);

    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    serde(&mut session)?;

    println!("Success");
    Ok(())
}
