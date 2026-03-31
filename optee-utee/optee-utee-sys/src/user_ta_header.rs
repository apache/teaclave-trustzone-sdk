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

use super::tee_api_types::*;
use super::utee_syscalls::*;
use super::utee_types::*;
use core::ffi::*;

pub const TA_FLAG_SINGLE_INSTANCE: u32 = 1 << 2;
pub const TA_FLAG_MULTI_SESSION: u32 = 1 << 3;
pub const TA_FLAG_INSTANCE_KEEP_ALIVE: u32 = 1 << 4;
pub const TA_FLAG_SECURE_DATA_PATH: u32 = 1 << 5;
pub const TA_FLAG_REMAP_SUPPORT: u32 = 1 << 6;
pub const TA_FLAG_CACHE_MAINTENANCE: u32 = 1 << 7;
pub const TA_FLAG_CONCURRENT: u32 = 1 << 8;
pub const TA_FLAG_DEVICE_ENUM: u32 = 1 << 9;
pub const TA_FLAG_DEVICE_ENUM_SUPP: u32 = 1 << 10;
pub const TA_FLAG_DONT_CLOSE_HANDLE_ON_CORRUPT_OBJECT: u32 = 1 << 11;
pub const TA_FLAG_DEVICE_ENUM_TEE_STORAGE_PRIVATE: u32 = 1 << 12;
pub const TA_FLAG_INSTANCE_KEEP_CRASHED: u32 = 1 << 13;

pub const TA_FLAG_EXEC_DDR: u32 = 0;
pub const TA_FLAG_USER_MODE: u32 = 0;
#[repr(C)]
pub struct ta_head {
    pub uuid: TEE_UUID,
    pub stack_size: u32,
    pub flags: u32,
    pub depr_entry: u64,
}

unsafe extern "C" {
    pub fn __utee_entry(
        func: c_ulong,
        session_id: c_ulong,
        up: *mut utee_params,
        cmd_id: c_ulong,
    ) -> TEE_Result;
}

/// # Safety
/// This function is the main entry point for a Trusted Application (TA) in OP-TEE.
/// It must only be called by the OP-TEE OS with valid parameters. The `up` parameter
/// is a raw pointer that must point to a valid `utee_params` structure initialized
/// by the OP-TEE runtime environment. This function should never be called directly
/// from user code - it is only exported for the OP-TEE OS loader.
#[unsafe(no_mangle)]
unsafe fn __ta_entry(
    func: c_ulong,
    session_id: c_ulong,
    up: *mut utee_params,
    cmd_id: c_ulong,
) -> ! {
    let res: u32 = unsafe { __utee_entry(func, session_id, up, cmd_id) };

    unsafe { _utee_return(res.into()) };
}

unsafe impl Sync for ta_head {}

pub const TA_PROP_STR_SINGLE_INSTANCE: *const c_char = c"gpd.ta.singleInstance".as_ptr();
pub const TA_PROP_STR_MULTI_SESSION: *const c_char = c"gpd.ta.multiSession".as_ptr();
pub const TA_PROP_STR_KEEP_ALIVE: *const c_char = c"gpd.ta.instanceKeepAlive".as_ptr();
pub const TA_PROP_STR_KEEP_CRASHED: *const c_char = c"gpd.ta.instanceKeepCrashed".as_ptr();
pub const TA_PROP_STR_DATA_SIZE: *const c_char = c"gpd.ta.dataSize".as_ptr();
pub const TA_PROP_STR_STACK_SIZE: *const c_char = c"gpd.ta.stackSize".as_ptr();
pub const TA_PROP_STR_VERSION: *const c_char = c"gpd.ta.version".as_ptr();
pub const TA_PROP_STR_DESCRIPTION: *const c_char = c"gpd.ta.description".as_ptr();
pub const TA_PROP_STR_ENDIAN: *const c_char = c"gpd.ta.endian".as_ptr();
pub const TA_PROP_STR_DOES_NOT_CLOSE_HANDLE_ON_CORRUPT_OBJECT: *const c_char =
    c"gpd.ta.doesNotCloseHandleOnCorruptObject".as_ptr();

#[repr(C)]
pub enum user_ta_prop_type {
    USER_TA_PROP_TYPE_BOOL,
    USER_TA_PROP_TYPE_U32,
    USER_TA_PROP_TYPE_UUID,
    USER_TA_PROP_TYPE_IDENTITY,
    USER_TA_PROP_TYPE_STRING,
    USER_TA_PROP_TYPE_BINARY_BLOCK,
    USER_TA_PROP_TYPE_U64,
    USER_TA_PROP_TYPE_INVALID,
}

#[repr(C)]
pub struct user_ta_property {
    pub name: *const c_char,
    pub prop_type: user_ta_prop_type,
    pub value: *const c_void,
}

unsafe impl Sync for user_ta_property {}
