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

extern crate alloc;
extern crate self as optee_utee;

pub type RawParamTypes = u32;
pub struct RawParams;
pub struct ParametersNone;

pub mod raw {
    #[allow(non_camel_case_types)]
    pub type TEE_Result = u32;
    pub const TEE_SUCCESS: TEE_Result = 0;
    pub const TEE_ERROR_SECURITY: TEE_Result = 0xffff000f;
}

pub struct Error;

impl Error {
    pub fn raw_code(&self) -> raw::TEE_Result {
        1
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait FromRawParameters<'a>: Sized {
    unsafe fn from_raw(_: RawParamTypes, _: &'a mut RawParams) -> Result<Self>;
}

impl<'a> FromRawParameters<'a> for ParametersNone {
    unsafe fn from_raw(_: RawParamTypes, _: &'a mut RawParams) -> Result<Self> {
        Ok(Self)
    }
}
