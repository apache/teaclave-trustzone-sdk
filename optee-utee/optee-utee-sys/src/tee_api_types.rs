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

// Common Definitions

use crate::libc_compat::size_t;
use core::ffi::*;

#[allow(non_camel_case_types)]
pub type TEE_Result = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TEE_UUID {
    pub timeLow: u32,
    pub timeMid: u16,
    pub timeHiAndVersion: u16,
    pub clockSeqAndNode: [u8; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TEE_Identity {
    pub login: u32,
    pub uuid: TEE_UUID,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Memref {
    pub buffer: *mut c_void,
    pub size: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Value {
    pub a: u32,
    pub b: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union TEE_Param {
    pub memref: Memref,
    pub value: Value,
}

#[allow(non_camel_case_types)]
pub type TEE_ParamType = u32;

#[allow(non_camel_case_types)]
pub type TEE_ParamTypes = u32;

#[allow(non_camel_case_types)]
pub type TEE_Params = [TEE_Param; 4];

#[repr(C)]
pub struct __TEE_TASessionHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_TASessionHandle = *mut __TEE_TASessionHandle;

#[repr(C)]
pub struct __TEE_PropSetHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_PropSetHandle = *mut __TEE_PropSetHandle;

#[repr(C)]
pub struct __TEE_ObjectHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_ObjectHandle = *mut __TEE_ObjectHandle;

#[repr(C)]
pub struct __TEE_ObjectEnumHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_ObjectEnumHandle = *mut __TEE_ObjectEnumHandle;

#[repr(C)]
pub struct __TEE_OperationHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_OperationHandle = *mut __TEE_OperationHandle;

// Storage Definitions

#[allow(non_camel_case_types)]
pub type TEE_ObjectType = u32;

#[repr(C)]
pub struct TEE_ObjectInfo {
    pub objectType: u32,
    pub objectSize: u32,
    pub maxObjectSize: u32,
    pub objectUsage: u32,
    pub dataSize: usize,
    pub dataPosition: usize,
    pub handleFlags: u32,
}

// Reserve the GP 1.1.1 type
#[repr(C)]
pub enum TEE_Whence {
    TEE_DATA_SEEK_SET,
    TEE_DATA_SEEK_CUR,
    TEE_DATA_SEEK_END,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union content {
    pub memref: Memref,
    pub value: Value,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TEE_Attribute {
    pub attributeID: u32,
    pub content: content,
}

// Cryptographic Operations API

// Reserve the GP 1.1.1 type
#[repr(C)]
pub enum TEE_OperationMode {
    TEE_MODE_ENCRYPT,
    TEE_MODE_DECRYPT,
    TEE_MODE_SIGN,
    TEE_MODE_VERIFY,
    TEE_MODE_MAC,
    TEE_MODE_DIGEST,
    TEE_MODE_DERIVE,
}

#[repr(C)]
pub struct TEE_OperationInfo {
    pub algorithm: u32,
    pub operationClass: u32,
    pub mode: u32,
    pub digestLength: u32,
    pub maxKeySize: u32,
    pub keySize: u32,
    pub requiredKeyUsage: u32,
    pub handleState: u32,
}

#[repr(C)]
pub struct TEE_OperationInfoKey {
    pub keySize: u32,
    pub requiredKeyUsage: u32,
}

#[repr(C)]
pub struct TEE_OperationInfoMultiple {
    pub algorithm: u32,
    pub operationClass: u32,
    pub mode: u32,
    pub digestLength: u32,
    pub maxKeySize: u32,
    pub handleState: u32,
    pub operationState: u32,
    pub numberOfKeys: u32,
    pub keyInformation: *mut TEE_OperationInfoKey,
}

// Time & Date API

#[repr(C)]
pub struct TEE_Time {
    pub seconds: u32,
    pub millis: u32,
}

// TEE Arithmetical APIs

#[allow(non_camel_case_types)]
pub type TEE_BigInt = u32;
#[allow(non_camel_case_types)]
pub type TEE_BigIntFMM = u32;
#[allow(non_camel_case_types)]
pub type TEE_BigIntFMMContext = u32;

// Tee Secure Element APIs

#[repr(C)]
pub struct __TEE_SEServiceHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_SEServiceHandle = *mut __TEE_SEServiceHandle;
#[repr(C)]
pub struct __TEE_SEReaderHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_SEReaderHandle = *mut __TEE_SEReaderHandle;
#[repr(C)]
pub struct __TEE_SESessionHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_SESessionHandle = *mut __TEE_SESessionHandle;
#[repr(C)]
pub struct __TEE_SEChannelHandle {
    _unused: [u8; 0],
}
#[allow(non_camel_case_types)]
pub type TEE_SEChannelHandle = *mut __TEE_SEChannelHandle;

#[repr(C)]
pub struct TEE_SEReaderProperties {
    pub sePresent: bool,
    pub teeOnly: bool,
    pub selectResponseEnable: bool,
}

#[repr(C)]
pub struct TEE_SEAID {
    pub buffer: *mut u8,
    pub bufferLen: size_t,
}

// Other definitions
#[allow(non_camel_case_types)]
pub type TEE_ErrorOrigin = u32;
#[allow(non_camel_case_types)]
pub type TEE_Session = *mut c_void;

pub const TEE_MEM_INPUT: u32 = 0x00000001;
pub const TEE_MEM_OUTPUT: u32 = 0x00000002;
pub const TEE_MEMREF_0_USED: u32 = 0x00000001;
pub const TEE_MEMREF_1_USED: u32 = 0x00000002;
pub const TEE_MEMREF_2_USED: u32 = 0x00000004;
pub const TEE_MEMREF_3_USED: u32 = 0x00000008;
pub const TEE_SE_READER_NAME_MAX: u32 = 20;
