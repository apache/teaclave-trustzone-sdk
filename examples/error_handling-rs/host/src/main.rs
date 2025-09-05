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

use optee_teec::ParamNone;
use optee_teec::{Context, ErrorKind, Operation, Result, Uuid};
use proto::{Command, UUID};

fn main() -> Result<()> {
    test_error_handling()
}

// NOTE: This seems to be a test case, not an example. Temporarily allow the expect
// here and the reorganization is planned for this kind of test cases.
#[allow(clippy::expect_used)]
fn test_error_handling() -> Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut session = ctx.open_session(uuid)?;
    let mut operation = Operation::new(0, ParamNone, ParamNone, ParamNone, ParamNone);

    // Test successful invocation return Ok().
    session
        .invoke_command(Command::ReturnSuccess as u32, &mut operation)
        .expect("success");

    // Test error invocation returns the requested error.
    let e = session
        .invoke_command(Command::ReturnGenericError as u32, &mut operation)
        .expect_err("generic error");
    assert_eq!(e.kind(), ErrorKind::Generic);

    // Test repeated error invocation also returns the requested error.
    let e = session
        .invoke_command(Command::ReturnGenericError as u32, &mut operation)
        .expect_err("generic error");
    assert_eq!(e.kind(), ErrorKind::Generic);

    println!("Test passed");
    Ok(())
}
