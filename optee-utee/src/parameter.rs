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

use crate::{Error, ErrorKind, Result};
use core::{marker, slice};
use optee_utee_sys as raw;

pub type RawParamType = u32;
pub type RawParamTypes = u32;
pub type RawParams = [raw::TEE_Param; 4];

pub struct Parameters(pub Parameter, pub Parameter, pub Parameter, pub Parameter);

impl Parameters {
    pub fn from_raw(tee_params: &mut RawParams, param_types: u32) -> Self {
        let (f0, f1, f2, f3) = ParamTypes::from(param_types).into_flags();
        let p0 = Parameter::from_raw(&mut tee_params[0], f0);
        let p1 = Parameter::from_raw(&mut tee_params[1], f1);
        let p2 = Parameter::from_raw(&mut tee_params[2], f2);
        let p3 = Parameter::from_raw(&mut tee_params[3], f3);

        Parameters(p0, p1, p2, p3)
    }
}

pub struct ParamValue<'parameter> {
    raw: *mut raw::Value,
    param_type: ParamType,
    _marker: marker::PhantomData<&'parameter mut u32>,
}

impl<'parameter> ParamValue<'parameter> {
    pub fn a(&self) -> u32 {
        unsafe { (*self.raw).a }
    }

    pub fn b(&self) -> u32 {
        unsafe { (*self.raw).b }
    }

    pub fn set_a(&mut self, a: u32) {
        unsafe {
            (*self.raw).a = a;
        }
    }

    pub fn set_b(&mut self, b: u32) {
        unsafe {
            (*self.raw).b = b;
        }
    }

    pub fn param_type(&self) -> ParamType {
        self.param_type
    }
}

pub struct ParamMemref<'parameter> {
    raw: *mut raw::Memref,
    param_type: ParamType,
    _marker: marker::PhantomData<&'parameter mut [u8]>,
}

impl<'parameter> ParamMemref<'parameter> {
    pub fn buffer(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut((*self.raw).buffer as *mut u8, (*self.raw).size) }
    }

    pub fn param_type(&self) -> ParamType {
        self.param_type
    }

    pub fn raw(&mut self) -> *mut raw::Memref {
        self.raw
    }

    pub fn set_updated_size(&mut self, size: usize) {
        unsafe { (*self.raw).size = size };
    }
}

pub struct Parameter {
    pub raw: *mut raw::TEE_Param,
    pub param_type: ParamType,
}

impl Parameter {
    pub fn from_raw(ptr: *mut raw::TEE_Param, param_type: ParamType) -> Self {
        Self {
            raw: ptr,
            param_type,
        }
    }

    /// # Safety
    /// The caller must ensure that the raw pointer is valid and points to a properly initialized TEE_Param.
    pub unsafe fn as_value(&mut self) -> Result<ParamValue> {
        match self.param_type {
            ParamType::ValueInput | ParamType::ValueInout | ParamType::ValueOutput => {
                Ok(ParamValue {
                    raw: &mut (*self.raw).value,
                    param_type: self.param_type,
                    _marker: marker::PhantomData,
                })
            }
            _ => Err(Error::new(ErrorKind::BadParameters)),
        }
    }

    /// # Safety
    /// The caller must ensure that the raw pointer is valid and points to a properly initialized TEE_Param.
    pub unsafe fn as_memref(&mut self) -> Result<ParamMemref> {
        match self.param_type {
            ParamType::MemrefInout | ParamType::MemrefInput | ParamType::MemrefOutput => {
                Ok(ParamMemref {
                    raw: &mut (*self.raw).memref,
                    param_type: self.param_type,
                    _marker: marker::PhantomData,
                })
            }
            _ => Err(Error::new(ErrorKind::BadParameters)),
        }
    }

    pub fn raw(&self) -> *mut raw::TEE_Param {
        self.raw
    }
}

pub struct ParamTypes(u32);

impl ParamTypes {
    pub fn into_flags(&self) -> (ParamType, ParamType, ParamType, ParamType) {
        (
            (0x000fu32 & self.0).into(),
            ((0x00f0u32 & self.0) >> 4).into(),
            ((0x0f00u32 & self.0) >> 8).into(),
            ((0xf000u32 & self.0) >> 12).into(),
        )
    }
}

impl From<u32> for ParamTypes {
    fn from(value: u32) -> Self {
        ParamTypes(value)
    }
}

#[derive(Copy, Clone)]
pub enum ParamType {
    None = 0,
    ValueInput = 1,
    ValueOutput = 2,
    ValueInout = 3,
    MemrefInput = 5,
    MemrefOutput = 6,
    MemrefInout = 7,
}

impl From<u32> for ParamType {
    fn from(value: u32) -> Self {
        match value {
            0 => ParamType::None,
            1 => ParamType::ValueInput,
            2 => ParamType::ValueOutput,
            3 => ParamType::ValueInout,
            5 => ParamType::MemrefInput,
            6 => ParamType::MemrefOutput,
            7 => ParamType::MemrefInout,
            _ => ParamType::None,
        }
    }
}
