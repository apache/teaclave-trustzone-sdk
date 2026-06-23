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

//! Type-safe wrappers for TEE value-type parameters.
//!
//! A *value* parameter carries two `u32` fields (`a` and `b`) rather than a
//! shared memory buffer.
//!
//! This module provides:
//!
//! * [`ParameterValueRead`] / [`ParameterValueWrite`] traits for reading and
//!   writing the two `u32` fields.
//! * Three concrete wrappers that encode the data direction in the type:
//!   [`ParameterValueInput`], [`ParameterValueOutput`],
//!   [`ParameterValueInout`].
//!
//! # Direction guarantees
//!
//! | Type | Host → TA | TA → Host |
//! |---|---|---|
//! | `ParameterValueInput` | ✓ | ✗ |
//! | `ParameterValueOutput` | ✗ | ✓ |
//! | `ParameterValueInout` | ✓ | ✓ |

use super::{FromRawParameter, ParamType, RawParamType, check_type_is};
use crate::{Result, raw::TEE_Param};

/// Read-only access to the two `u32` fields of a value parameter.
///
/// Implemented by [`ParameterValueInput`] and [`ParameterValueInout`].
pub trait ParameterValueRead {
    /// Returns the `a` field.
    fn get_a(&self) -> u32;
    /// Returns the `b` field.
    fn get_b(&self) -> u32;
}

/// Write access to the two `u32` fields of a value parameter.
///
/// Implemented by [`ParameterValueOutput`] and [`ParameterValueInout`].
/// Values written here will be read back by the host after the TA invocation
/// completes.
pub trait ParameterValueWrite {
    /// Set the `a` field.
    fn set_a(&mut self, a: u32);
    /// Set the `b` field.
    fn set_b(&mut self, b: u32);
}

/// A value-type input parameter.
///
/// The two `u32` values are copied out of the raw union at construction time,
pub struct ParameterValueInput {
    a: u32,
    b: u32,
}

/// A value-type output parameter.
///
/// Holds a mutable reference into the raw `TEE_Param` union. Values written
/// via [`ParameterValueWrite`] will be propagated back to the host.
pub struct ParameterValueOutput<'a>(&'a mut TEE_Param);

/// A value-type in/out parameter.
///
/// Combines the semantics of both [`ParameterValueInput`] and
/// [`ParameterValueOutput`]: the host initializes the values, and the TA may
/// both read and write them.
pub struct ParameterValueInout<'a>(&'a mut TEE_Param);

impl<'a> FromRawParameter<'a> for ParameterValueInput {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::ValueInput)?;
        Ok(Self {
            a: unsafe { raw_param.value.a },
            b: unsafe { raw_param.value.b },
        })
    }
}

impl<'a> FromRawParameter<'a> for ParameterValueOutput<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::ValueOutput)?;
        Ok(Self(raw_param))
    }
}

impl<'a> FromRawParameter<'a> for ParameterValueInout<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::ValueInout)?;
        Ok(Self(raw_param))
    }
}

impl ParameterValueRead for ParameterValueInput {
    fn get_a(&self) -> u32 {
        self.a
    }
    fn get_b(&self) -> u32 {
        self.b
    }
}

impl<'a> ParameterValueRead for ParameterValueInout<'a> {
    fn get_a(&self) -> u32 {
        unsafe { self.0.value.a }
    }
    fn get_b(&self) -> u32 {
        unsafe { self.0.value.b }
    }
}

impl<'a> ParameterValueWrite for ParameterValueInout<'a> {
    fn set_a(&mut self, a: u32) {
        self.0.value.a = a;
    }
    fn set_b(&mut self, b: u32) {
        self.0.value.b = b;
    }
}

impl<'a> ParameterValueWrite for ParameterValueOutput<'a> {
    fn set_a(&mut self, a: u32) {
        self.0.value.a = a;
    }
    fn set_b(&mut self, b: u32) {
        self.0.value.b = b;
    }
}
