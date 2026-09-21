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

use num_enum::{FromPrimitive, IntoPrimitive};

mod in_out;
pub use in_out::*;


#[derive(FromPrimitive, IntoPrimitive, Debug)]
#[repr(u32)]
pub enum Command {
    CreateWallet,
    RemoveWallet,
    DeriveAddress,
    SignTransaction,
    #[default]
    Unknown,
}

pub const UUID: &str = "be2dc9a0-02b4-4b33-ba21-9964dbdf1573";
