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

use libc::*;

pub fn TEEC_PARAM_TYPES(p0:u32, p1:u32, p2:u32, p3:u32) -> u32 {
    let tmp = p1 << 4 | p2 << 8 | p3 << 12;
    p0 | tmp
}

pub const TEEC_CONFIG_PAYLOAD_REF_COUNT: u32 = 4;

pub const TEEC_CONFIG_SHAREDMEM_MAX_SIZE: c_ulong = -1 as c_long as c_ulong;

pub const TEEC_NONE: u32                  = 0x00000000;
pub const TEEC_VALUE_INPUT: u32           = 0x00000001;
pub const TEEC_VALUE_OUTPUT: u32          = 0x00000002;
pub const TEEC_VALUE_INOUT: u32           = 0x00000003;
pub const TEEC_MEMREF_TEMP_INPUT: u32     = 0x00000005;
pub const TEEC_MEMREF_TEMP_OUTPUT: u32    = 0x00000006;
pub const TEEC_MEMREF_TEMP_INOUT: u32     = 0x00000007;
pub const TEEC_MEMREF_WHOLE: u32          = 0x0000000C;
pub const TEEC_MEMREF_PARTIAL_INPUT: u32  = 0x0000000D;
pub const TEEC_MEMREF_PARTIAL_OUTPUT: u32 = 0x0000000E;
pub const TEEC_MEMREF_PARTIAL_INOUT: u32  = 0x0000000F;

pub const TEEC_MEM_INPUT: u32  = 0x00000001;
pub const TEEC_MEM_OUTPUT: u32 = 0x00000002;

pub const TEEC_SUCCESS: u32               = 0x00000000;
pub const TEEC_ERROR_GENERIC: u32         = 0xFFFF0000;
pub const TEEC_ERROR_ACCESS_DENIED: u32   = 0xFFFF0001;
pub const TEEC_ERROR_CANCEL: u32          = 0xFFFF0002;
pub const TEEC_ERROR_ACCESS_CONFLICT: u32 = 0xFFFF0003;
pub const TEEC_ERROR_EXCESS_DATA: u32     = 0xFFFF0004;
pub const TEEC_ERROR_BAD_FORMAT: u32      = 0xFFFF0005;
pub const TEEC_ERROR_BAD_PARAMETERS: u32  = 0xFFFF0006;
pub const TEEC_ERROR_BAD_STATE: u32       = 0xFFFF0007;
pub const TEEC_ERROR_ITEM_NOT_FOUND: u32  = 0xFFFF0008;
pub const TEEC_ERROR_NOT_IMPLEMENTED: u32 = 0xFFFF0009;
pub const TEEC_ERROR_NOT_SUPPORTED: u32   = 0xFFFF000A;
pub const TEEC_ERROR_NO_DATA: u32         = 0xFFFF000B;
pub const TEEC_ERROR_OUT_OF_MEMORY: u32   = 0xFFFF000C;
pub const TEEC_ERROR_BUSY: u32            = 0xFFFF000D;
pub const TEEC_ERROR_COMMUNICATION: u32   = 0xFFFF000E;
pub const TEEC_ERROR_SECURITY: u32        = 0xFFFF000F;
pub const TEEC_ERROR_SHORT_BUFFER: u32    = 0xFFFF0010;
pub const TEEC_ERROR_EXTERNAL_CANCEL: u32 = 0xFFFF0011;
pub const TEEC_ERROR_TARGET_DEAD: u32     = 0xFFFF3024;

pub const TEEC_ORIGIN_API: u32         = 0x00000001;
pub const TEEC_ORIGIN_COMMS: u32       = 0x00000002;
pub const TEEC_ORIGIN_TEE: u32         = 0x00000003;
pub const TEEC_ORIGIN_TRUSTED_APP: u32 = 0x00000004;

pub const TEEC_LOGIN_PUBLIC: u32            = 0x00000000;
pub const TEEC_LOGIN_USER: u32              = 0x00000001;
pub const TEEC_LOGIN_GROUP: u32             = 0x00000002;
pub const TEEC_LOGIN_APPLICATION: u32       = 0x00000004;
pub const TEEC_LOGIN_USER_APPLICATION: u32  = 0x00000005;
pub const TEEC_LOGIN_GROUP_APPLICATION: u32 = 0x00000006;

#[allow(non_camel_case_types)]
pub type TEEC_Result = u32;

#[repr(C)]
pub struct TEEC_Context__Imp {
    pub fd: c_int,
    pub reg_mem: bool,
    pub memref_null: bool,
}

#[repr(C)]
pub struct TEEC_Context {
    pub imp: TEEC_Context__Imp,
}

#[repr(C)]
pub struct TEEC_UUID {
    pub timeLow: u32,
    pub timeMid: u16,
    pub timeHiAndVersion: u16,
    pub clockSeqAndNode: [u8; 8],
}

#[repr(C)]
pub struct TEEC_Session__Imp {
    pub ctx: *mut TEEC_Context,
    pub session_id: u32,
}

#[repr(C)]
pub struct TEEC_Session {
    pub imp: TEEC_Session__Imp,
}

#[repr(C)]
pub struct TEEC_SharedMemory__Imp {
    pub id: c_int,
    pub alloced_size: size_t,
    pub shadow_buffer: *mut c_void,
    pub registered_fd: c_int,
    pub flags: u32,
}

#[repr(C)]
pub struct TEEC_SharedMemory {
    pub buffer: *mut c_void,
    pub size: size_t,
    pub flags: u32,
    pub imp: TEEC_SharedMemory__Imp,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TEEC_TempMemoryReference {
    pub buffer: *mut c_void,
    pub size: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TEEC_RegisteredMemoryReference {
    pub parent: *mut TEEC_SharedMemory,
    pub size: size_t,
    pub offset: size_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TEEC_Value {
    pub a: u32,
    pub b: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union TEEC_Parameter {
    pub tmpref: TEEC_TempMemoryReference,
    pub memref: TEEC_RegisteredMemoryReference,
    pub value: TEEC_Value,
}

#[repr(C)]
pub struct TEEC_Operation__Imp {
    pub session: *mut TEEC_Session,
}

#[repr(C)]
pub struct TEEC_Operation {
    pub started: u32,
    pub paramTypes: u32,
    pub params: [TEEC_Parameter; TEEC_CONFIG_PAYLOAD_REF_COUNT as usize],
    pub imp: TEEC_Operation__Imp,
}

extern "C" {
    pub fn TEEC_InitializeContext(name: *const c_char, context: *mut TEEC_Context) -> TEEC_Result;
    pub fn TEEC_FinalizeContext(context: *mut TEEC_Context);
    pub fn TEEC_OpenSession(context: *mut TEEC_Context,
                            session: *mut TEEC_Session,
                            destination: *const TEEC_UUID,
                            connectionMethod: u32,
                            connectionData: *const c_void,
                            operation: *mut TEEC_Operation,
                            returnOrigin: *mut u32) -> TEEC_Result;
    pub fn TEEC_CloseSession(session: *mut TEEC_Session);
    pub fn TEEC_InvokeCommand(session: *mut TEEC_Session,
                              commandID: u32,
                              operation: *mut TEEC_Operation,
                              returnOrigin: *mut u32) -> TEEC_Result;
    pub fn TEEC_RegisterSharedMemory(context: *mut TEEC_Context,
                                     sharedMem: *mut TEEC_SharedMemory) -> TEEC_Result;
    pub fn TEEC_AllocateSharedMemory(context: *mut TEEC_Context,
                                     sharedMem: *mut TEEC_SharedMemory) -> TEEC_Result;
    pub fn TEEC_ReleaseSharedMemory(sharedMemory: *mut TEEC_SharedMemory);
    pub fn TEEC_RequestCancellation(operation: *mut TEEC_Operation);
}
