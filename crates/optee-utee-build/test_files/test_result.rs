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

use core::ffi::*;
use core::mem;
use core::primitive::u64;
const TA_FLAGS: u32 = 0u32;
const TA_DATA_SIZE: u32 = 32768u32;
const TA_STACK_SIZE: u32 = 2048u32;
const TA_VERSION: &[u8] = b"0.1.0\0";
const TA_DESCRIPTION: &[u8] = b"test\0";
#[unsafe(no_mangle)]
pub static mut trace_level: c_int = 4i32;
#[unsafe(no_mangle)]
pub static trace_ext_prefix: [u8; 3usize] = *b"TA\0";
/// # Safety
/// This function is called by the OP-TEE framework to get the trace level.
/// It's safe to call as it only reads a static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tahead_get_trace_level() -> c_int {
    unsafe { trace_level }
}
/// The TA property table exported as `ta_props`.
///
/// `libutee` reads it through an `extern const struct user_ta_property
/// ta_props[]`, so it must be a `static` with exactly this layout.
/// `user_ta_property` holds raw pointers and is therefore not `Sync` on
/// its own; every pointer in the table points to immutable data in the
/// TA image (`&'static` property names and values) and nothing ever
/// writes through it, so sharing the table is sound.
#[repr(transparent)]
pub struct TaProperties<const N: usize>([optee_utee::raw::user_ta_property; N]);
unsafe impl<const N: usize> Sync for TaProperties<N> {}
const IS_SINGLE_INSTANCE: bool = (TA_FLAGS & optee_utee::raw::TA_FLAG_SINGLE_INSTANCE)
    != 0;
const IS_MULTI_SESSION: bool = (TA_FLAGS & optee_utee::raw::TA_FLAG_MULTI_SESSION) != 0;
const IS_KEEP_ALIVE: bool = (TA_FLAGS & optee_utee::raw::TA_FLAG_INSTANCE_KEEP_ALIVE)
    != 0;
const IS_KEEP_CRASHED: bool = (TA_FLAGS & optee_utee::raw::TA_FLAG_INSTANCE_KEEP_CRASHED)
    != 0;
const TA_ENDIAN: u32 = 0;
const DONT_CLOSE_HANDLE_ON_CORRUPT_OBJECT: bool = (TA_FLAGS
    & optee_utee::raw::TA_FLAG_DONT_CLOSE_HANDLE_ON_CORRUPT_OBJECT) != 0;
#[unsafe(no_mangle)]
pub static ta_num_props: usize = 10usize;
#[unsafe(no_mangle)]
pub static ta_props: TaProperties<10usize> = TaProperties([
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_SINGLE_INSTANCE.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &IS_SINGLE_INSTANCE as *const bool as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_MULTI_SESSION.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &IS_MULTI_SESSION as *const bool as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_KEEP_ALIVE.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &IS_KEEP_ALIVE as *const bool as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_KEEP_CRASHED.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &IS_KEEP_CRASHED as *const bool as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_DATA_SIZE.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_DATA_SIZE as *const u32 as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_STACK_SIZE.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_STACK_SIZE as *const u32 as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_VERSION.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_VERSION as *const [u8] as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_DESCRIPTION.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
        value: TA_DESCRIPTION as *const [u8] as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_ENDIAN.as_ptr().cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
        value: &TA_ENDIAN as *const u32 as _,
    },
    optee_utee::raw::user_ta_property {
        name: optee_utee::raw::TA_PROP_STR_DOES_NOT_CLOSE_HANDLE_ON_CORRUPT_OBJECT
            .as_ptr()
            .cast(),
        type_: optee_utee::raw::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
        value: &DONT_CLOSE_HANDLE_ON_CORRUPT_OBJECT as *const bool as _,
    },
]);
#[unsafe(no_mangle)]
#[unsafe(link_section = ".ta_head")]
pub static ta_head: optee_utee::raw::ta_head = optee_utee::raw::ta_head {
    uuid: optee_utee::raw::TEE_UUID {
        timeLow: 642817260u32,
        timeMid: 18987u16,
        timeHiAndVersion: 18741u16,
        clockSeqAndNode: [135u8, 171u8, 118u8, 45u8, 137u8, 251u8, 240u8, 176u8],
    },
    stack_size: 4096u32,
    flags: TA_FLAGS,
    depr_entry: u64::MAX,
};
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss")]
pub static mut ta_heap: [u8; TA_DATA_SIZE as usize] = [0; TA_DATA_SIZE as usize];
#[unsafe(no_mangle)]
pub static ta_heap_size: usize = mem::size_of::<u8>() * TA_DATA_SIZE as usize;
