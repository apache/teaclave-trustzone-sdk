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

//! Build-time helper for OP-TEE supplicant plugins.
//!
//! This crate provides [`PluginConfig`] which generates a Rust source file
//! containing the static `plugin_method` symbol required by the OP-TEE
//! plugin ABI. It is intended to be used in a `build.rs` script.

mod plugin;
pub use uuid;

pub use plugin::{DEFAULT_INIT_FN_NAME, DEFAULT_INVOKE_FN_NAME, PluginConfig};
