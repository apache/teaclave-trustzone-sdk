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

#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum Command {
    Ping,
    #[default]
    Unknown,
}

pub const TA_UUID: &str = "255fc838-de89-42d3-9a8e-d044c50fa57c";

//for plugin
#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum PluginCommand {
    Print,
    #[default]
    Unknown,
}

pub const PLUGIN_SUBCMD_NULL: u32 = 0xFFFFFFFF;
pub const PLUGIN_UUID: &str = "ef620757-fa2b-4f19-a1c4-6e51cfe4c0f9";
