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

use super::*;

#[repr(C)]
pub enum utee_time_category {
    UTEE_TIME_CAT_SYSTEM,
    UTEE_TIME_CAT_TA_PERSISTENT,
    UTEE_TIME_CAT_REE,
}

#[repr(C)]
pub enum utee_entry_func {
    UTEE_ENTRY_FUNC_OPEN_SESSION,
    UTEE_ENTRY_FUNC_CLOSE_SESSION,
    UTEE_ENTRY_FUNC_INVOKE_COMMAND,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum utee_cache_operation {
    TEE_CACHECLEAN,
    TEE_CACHEFLUSH,
    TEE_CACHEINVALIDATE,
}

#[repr(C)]
pub struct utee_params {
    types: u64,
    vals: [u64; TEE_NUM_PARAMS as usize * 2],
}

#[repr(C)]
pub struct utee_attribute {
    a: u64,
    b: u64,
    attribute_id: u32,
}

#[repr(C)]
pub struct utee_object_info {
    obj_type: u32,
    obj_size: u32,
    max_obj_size: u32,
    obj_usage: u32,
    data_size: u32,
    data_pos: u32,
    handle_flags: u32,
}
