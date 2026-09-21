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
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum Command {
    Train,
    Valid,
    Export,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Output {
    pub loss: f32,
    pub accuracy: f32,
}

pub const UUID: &str = "1b5f5b74-e9cf-4e62-8c3e-7e41da6d76f6";
