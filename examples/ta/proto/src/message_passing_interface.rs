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

extern crate alloc;

use alloc::string::String;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, FromPrimitive, Debug, Copy, Clone)]
#[repr(u32)]
pub enum Command {
    Hello,
    Bye,
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnclaveInput {
    pub command: Command,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnclaveOutput {
    pub message: String,
}

pub const UUID: &str = "17556a46-bdab-11eb-b325-d38c9a9af725";
