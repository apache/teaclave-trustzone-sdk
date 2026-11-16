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

use optee_teec::{Context, ErrorKind, Operation, ParamNone, Uuid};
use proto::inter_ta::{Command, UUID};

fn main() -> optee_teec::Result<()> {
    let ctx = Context::new()?;
    let uuid =
        Uuid::parse_str(UUID).map_err(|_| optee_teec::Error::from(ErrorKind::BadParameters))?;
    let mut session = ctx.open_session(uuid)?;
    let mut operation = Operation::new(0, ParamNone, ParamNone, ParamNone, ParamNone);

    // Nothing to send, just invoke the Test command
    session.invoke_command(Command::Test as u32, &mut operation)?;
    println!("Success");
    Ok(())
}
