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

include!("support.rs");

use optee_utee_macros::{ta_close_session, ta_invoke_command, ta_open_session};

#[derive(Default)]
struct SessionContext;

#[ta_open_session]
fn open_session(_: &mut ParametersNone, _: &mut SessionContext) -> Result<()> {
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(_: u32, _: &mut ParametersNone) -> Result<()> {
    Ok(())
}

#[ta_close_session]
fn close_session(_: &mut SessionContext) {}

fn main() {}
