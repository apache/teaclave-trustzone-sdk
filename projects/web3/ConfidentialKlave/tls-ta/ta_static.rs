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

#[no_mangle]
pub static mut trace_level: libc::c_int = TRACE_LEVEL;

#[no_mangle]
pub static trace_ext_prefix: &[u8] = TRACE_EXT_PREFIX;

#[no_mangle]
#[link_section = ".ta_head"]
pub static ta_head: optee_utee_sys::ta_head = optee_utee_sys::ta_head {
    uuid: TA_UUID,
    stack_size: TA_STACK_SIZE + TA_FRAMEWORK_STACK_SIZE,
    flags: TA_FLAGS,
    depr_entry: std::u64::MAX,
};

#[no_mangle]
#[link_section = ".bss"]
pub static ta_heap: [u8; TA_DATA_SIZE as usize] = [0; TA_DATA_SIZE as usize];

#[no_mangle]
pub static ta_heap_size: libc::size_t = std::mem::size_of::<u8>() * TA_DATA_SIZE as usize;
static FLAG_BOOL: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_SINGLE_INSTANCE) != 0;
static FLAG_MULTI: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_MULTI_SESSION) != 0;
static FLAG_INSTANCE: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_INSTANCE_KEEP_ALIVE) != 0;

#[no_mangle]
pub static ta_num_props: libc::size_t = 9;

#[no_mangle]
pub static ta_props: [optee_utee_sys::user_ta_property; 9] = [
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_SINGLE_INSTANCE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_BOOL as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_MULTI_SESSION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_MULTI as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_KEEP_ALIVE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &FLAG_INSTANCE as *const bool as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_DATA_SIZE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_DATA_SIZE as *const u32 as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_STACK_SIZE,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_STACK_SIZE as *const u32 as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_VERSION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_VERSION as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: optee_utee_sys::TA_PROP_STR_DESCRIPTION,
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_DESCRIPTION as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: "gp.ta.description\0".as_ptr(),
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: EXT_PROP_VALUE_1 as *const [u8] as *mut _,
    },
    optee_utee_sys::user_ta_property {
        name: "gp.ta.version\0".as_ptr(),
        prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &EXT_PROP_VALUE_2 as *const u32 as *mut _,
    },
];

#[no_mangle]
pub unsafe extern "C" fn tahead_get_trace_level() -> libc::c_int {
    return trace_level;
}
