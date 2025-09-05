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

use optee_teec::{Context, Operation, ParamType, Session, Uuid};
use optee_teec::{ParamNone, ParamValue};
use proto::{Command, UUID};

fn hello_world(session: &mut Session) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(29, 0, ParamType::ValueInout);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    println!("original value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::IncValue as u32, &mut operation)?;
    println!("inc value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::DecValue as u32, &mut operation)?;
    println!("dec value is {:?}", operation.parameters().0.a());
    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;

    hello_world(&mut session)?;

    println!("Success");
    Ok(())
}
