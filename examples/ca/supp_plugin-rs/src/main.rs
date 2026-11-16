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
use proto::supp_plugin::{Command, TA_UUID};

fn ping_ta(session: &mut Session) -> optee_teec::Result<()> {
    let test_data = [0x36u8; 10];
    let p0 = ParamTmpRef::new_input(&test_data);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    println!("-*Work logic: host -> TA -> plugin*-");
    println!("*host*: send value {:?} to ta", test_data);
    session.invoke_command(Command::Ping as u32, &mut operation)?;
    println!("*host*: invoke commmand finished");

    Ok(())
}

fn main() -> optee_teec::Result<()> {
    let ctx = Context::new()?;
    let uuid = Uuid::parse_str(TA_UUID).map_err(|err| {
        println!("Invalid TA_UUID: {:?}", err);
        ErrorKind::BadParameters
    })?;
    let mut session = ctx.open_session(uuid)?;

    ping_ta(&mut session)?;

    println!("Success");
    Ok(())
}
