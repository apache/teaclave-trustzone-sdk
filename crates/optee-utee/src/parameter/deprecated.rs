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

//! Legacy raw-pointer-based parameter API (deprecated).
//!
//! The types in this module use raw `*mut TEE_Param` pointers and require
//! manual unsafe casts at every call site. **New code should use the typed
//! wrappers** instead:
//!
//! | Legacy type | Replacement |
//! |---|---|
//! | [`Parameters`] | Use [`FromRawParameters`] with a 4-tuple of typed wrappers (e.g. `(ParameterNone, ParameterMemrefInput, ParameterValueOutput, ParameterMemrefOutput)`) |
//! | [`Parameter`] | Use the concrete types: [`ParameterValueInput`](crate::ParameterValueInput), [`ParameterValueOutput`](crate::ParameterValueOutput), [`ParameterMemrefInput`](crate::ParameterMemrefInput), etc. |
//! | [`ParamValue`] | Use the concrete types: [`ParameterValueInput`](crate::ParameterValueInput), [`ParameterValueOutput`](crate::ParameterValueOutput), [`ParameterValueInout`](crate::ParameterValueInout), and the traits: [`ParameterValueRead`](crate::ParameterValueRead) and [`ParameterValueWrite`](crate::ParameterValueWrite) |
//! | [`ParamMemref`] | Use the concrete types: [`ParameterMemrefInput`](crate::ParameterMemrefInput), [`ParameterMemrefOutput`](crate::ParameterMemrefOutput), [`ParameterMemrefInout`](crate::ParameterMemrefInout), and the traits: [`ParameterMemrefRead`](crate::ParameterMemrefRead) and [`ParameterMemrefWrite`](crate::ParameterMemrefWrite) |
//!
//! # Example migration
//!
//! **Old (deprecated) code:**
//!
//! ```rust,ignore
//! fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
//!     let mut p0 = unsafe { params.0.as_memref()? };
//!     // extra codes here...
//!     let data = p0.buffer();
//!     data.copy_from_slice(&output);
//!     p0.set_updated_size(output.len());
//! }
//! ```
//!
//! **New code:**
//!
//! ```rust,ignore
//! use optee_utee::prelude::*;
//!
//! fn invoke_command(cmd_id: u32, params: &mut (ParameterMemrefInout, ...)) -> Result<()> {
//!     let p0 = &mut params.0;
//!     // extra codes here...
//!     p0.write_at(0, output)
//! }
//! ```

use super::{ParamType, RawParamTypes, RawParams};
use crate::{Error, ErrorKind, FromRawParameters, Result, raw};
use core::{marker, slice};

/// # Deprecated
///
/// Use the typed wrappers with [`super::FromRawParameters`] instead. See the
/// [module-level documentation](self) for migration examples.
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

/// # Deprecated
///
/// Use [`super::value::ParameterValueRead`] (for `get_a/get_b`) and
/// [`super::value::ParameterValueWrite`] (for `set_a/set_b`) on the typed
/// wrappers ([`super::value::ParameterValueInput`], etc.) instead.
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

/// Lightweight accessor for a memory-reference TEE parameter.
///
/// # Deprecated
///
/// Use [`super::memref::ParameterMemrefRead`] (for `get_buffer`) and
/// [`super::memref::ParameterMemrefWrite`] (for `get_buffer_mut`,
/// `set_updated_size`, `write_at`) on the typed wrappers instead.
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

/// # Deprecated
///
/// Use the concrete typed wrappers
/// ([`super::value::ParameterValueInput`], [`super::memref::ParameterMemrefInput`], etc.)
/// instead. They encode the parameter type in the Rust type system, making
/// mismatched-type errors impossible at compile time.
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
    ///
    /// The caller must ensure that the raw pointer is valid and points to a
    /// properly initialized `TEE_Param`.
    pub unsafe fn as_value(&mut self) -> Result<ParamValue<'_>> {
        match self.param_type {
            ParamType::ValueInput | ParamType::ValueInout | ParamType::ValueOutput => {
                Ok(ParamValue {
                    raw: unsafe { &mut (*self.raw).value },
                    param_type: self.param_type,
                    _marker: marker::PhantomData,
                })
            }
            _ => Err(Error::new(ErrorKind::BadParameters)),
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the raw pointer is valid and points to a
    /// properly initialized `TEE_Param`.
    pub unsafe fn as_memref(&mut self) -> Result<ParamMemref<'_>> {
        match self.param_type {
            ParamType::MemrefInout | ParamType::MemrefInput | ParamType::MemrefOutput => {
                Ok(ParamMemref {
                    raw: unsafe { &mut (*self.raw).memref },
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

/// # Deprecated
///
/// Use the typed wrappers with [`super::FromRawParameters`] instead; they
/// extract the types implicitly via `TEE_PARAM_TYPE_GET`.
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

impl<'a> FromRawParameters<'a> for Parameters {
    unsafe fn from_raw(raw_types: RawParamTypes, raw_params: &'a mut RawParams) -> Result<Self> {
        Ok(Self::from_raw(raw_params, raw_types))
    }
}
