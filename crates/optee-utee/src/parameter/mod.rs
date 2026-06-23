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

//! TEE parameter handling: conversion from raw OP-TEE parameter types into
//! type-safe Rust wrappers.
//!
//! # Overview
//!
//! OP-TEE passes up to four parameters between the host (Normal World) and
//! the TA (Trusted Application). Each parameter has an associated *type* tag
//! (e.g. value, memref) and a raw `TEE_Param` union. This module provides:
//!
//! * **`FromRawParameter`** – single-parameter conversion from a raw
//!   `TEE_Param` into a typed Rust wrapper.
//! * **`FromRawParameters`** – batch conversion of all four parameters at once
//!   (implemented for 4-tuples of `FromRawParameter` types).
//! * **Typed wrappers** – `ParameterNone`, `ParameterValueInput`,
//!   `ParameterMemrefInput`, etc.
//! * **Type-erased wrapper** – [`ParameterAny`] for scenarios where the
//!   developer cannot know the parameter type at compile time.
//! * **Legacy compatibility** – [`deprecated`] provides the old unsafe
//!   pointer-based API; new code should use the typed wrappers instead.
//!
//! # Migration from the deprecated API
//!
//! If you currently use [`deprecated::Parameters`] and [`deprecated::Parameter`]:
//!
//! | Old pattern | New pattern |
//! |---|---|
//! | `params.0.as_value()?.a()` | `param.get_a()` via [`ParameterValueRead`](crate::ParameterValueRead] |
//! | `params.0.as_memref()?.buffer()` | `param.get_buffer()` or `param.get_buffer_mut()` via [`crate::ParameterMemrefRead`]/[`crate::ParameterMemrefWrite`] |
//! | `params.0.set_updated_size(n)` | `param.set_updated_size(n)` via [`crate::ParameterMemrefWrite`] |
//!
//! See the [`deprecated`] module for per-type migration notes.

use crate::{ErrorKind, Result, raw};

pub mod deprecated;
pub mod memref;
pub mod none;
pub mod value;

/// Raw parameter-type tag as passed by the TEE runtime.
/// Each of the four slots carries a 4-bit type-identifier. Use
/// `TEE_PARAM_TYPE_GET(raw_types, idx)` to extract one slot from
/// the `RawParamTypes` bit-field.
pub type RawParamType = u32;

/// Together with a [`RawParams`] array this describes the name *and* content
/// of all four TEE parameters.
pub type RawParamTypes = u32;

/// Array of four raw `TEE_Param` unions.
///
/// Corresponds to the four parameter slots passed to every TA entry-point.
pub type RawParams = [raw::TEE_Param; raw::TEE_NUM_PARAMS as usize];

/// Convert a single raw parameter into a type-safe Rust wrapper.
///
/// # Safety
///
/// Implementors must validate that `raw_type` matches the expected parameter
/// type and that `raw_param` has been correctly initialized by the TEE
/// runtime before reading any fields.
pub trait FromRawParameter<'a>: Sized {
    /// Construct `Self` from a raw parameter.
    ///
    /// # Safety
    ///
    /// Caller must ensure `raw_type` and `raw_param` are a consistent pair
    /// delivered by a TEE entry-point invocation (e.g. `TA_InvokeCommandEntryPoint`).
    #[allow(clippy::missing_safety_doc)]
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut raw::TEE_Param) -> Result<Self>;
}

/// Convert all four raw parameters at once into a typed tuple.
///
/// # Safety
///
/// The same safety constraints as [`FromRawParameter`] apply to each slot.
pub trait FromRawParameters<'a>: Sized {
    /// Construct `Self` from the full parameter array.
    ///
    /// # Safety
    ///
    /// Caller must ensure `raw_types` and `raw_param` come from the same
    /// TEE entry-point invocation.
    #[allow(clippy::missing_safety_doc)]
    unsafe fn from_raw(raw_types: RawParamTypes, raw_params: &'a mut RawParams) -> Result<Self>;
}

impl<
    'a,
    A: FromRawParameter<'a>,
    B: FromRawParameter<'a>,
    C: FromRawParameter<'a>,
    D: FromRawParameter<'a>,
> FromRawParameters<'a> for (A, B, C, D)
{
    unsafe fn from_raw(raw_types: RawParamTypes, raw_params: &'a mut RawParams) -> Result<Self> {
        Ok(unsafe {
            let [p0, p1, p2, p3] = raw_params;
            (
                A::from_raw(raw::TEE_PARAM_TYPE_GET(raw_types, 0), p0)?,
                B::from_raw(raw::TEE_PARAM_TYPE_GET(raw_types, 1), p1)?,
                C::from_raw(raw::TEE_PARAM_TYPE_GET(raw_types, 2), p2)?,
                D::from_raw(raw::TEE_PARAM_TYPE_GET(raw_types, 3), p3)?,
            )
        })
    }
}

/// Enumerates the seven standard TEE parameter types.
///
/// This is the Rust-side mirror of the `TEE_PARAM_TYPE_*` constants defined
/// in the C header. The `Unknown(u32)` variant catches any
/// implementation-defined or invalid type tags.
#[derive(Copy, Clone, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(u32)]
pub enum ParamType {
    None = raw::TEE_PARAM_TYPE_NONE,
    ValueInput = raw::TEE_PARAM_TYPE_VALUE_INPUT,
    ValueOutput = raw::TEE_PARAM_TYPE_VALUE_OUTPUT,
    ValueInout = raw::TEE_PARAM_TYPE_VALUE_INOUT,
    MemrefInput = raw::TEE_PARAM_TYPE_MEMREF_INPUT,
    MemrefOutput = raw::TEE_PARAM_TYPE_MEMREF_OUTPUT,
    MemrefInout = raw::TEE_PARAM_TYPE_MEMREF_INOUT,
    #[num_enum(catch_all)]
    Unknown(u32),
}

fn check_type_is(raw_type: RawParamType, exp_type: ParamType) -> Result<()> {
    let exp_type: u32 = exp_type.into();
    if raw_type != exp_type {
        return Err(ErrorKind::BadParameters.into());
    }
    Ok(())
}

/// A type-erased parameter that dispatches to the correct concrete wrapper
/// based on the runtime type tag.
///
/// # When to use
///
/// Use `ParameterAny` when the parameter type is **not known at compile time**
/// and must be determined at runtime. Typical scenarios:
///
/// * The command semantics allow multiple parameter types for the same slot
///   (e.g. "slot 0 may be `None` or `MemrefInput` depending on the command").
/// * A library helper is designed to inspect parameters generically without
///   fixing the types in its signature.
///
/// If you already know the expected type for a slot, prefer the concrete
/// wrappers from [`value`] and [`memref`] instead—they provide methods
/// specific to that type and avoid the match ceremony.
///
/// # Example
///
/// ```rust,ignore
/// use optee_utee::parameter::{ParameterAny, FromRawParameter};
///
/// match param {
///     ParameterAny::None => { /* no parameter */ }
///     ParameterAny::MemrefInput(p) => {
///         let data: &[u8] = p.get_buffer();
///         // process data ...
///     }
///     ParameterAny::ValueInput(p) => {
///         let a = p.get_a();
///         let b = p.get_b();
///         // use a, b ...
///     }
///     _ => return Err(ErrorKind::BadParameters.into()),
/// }
/// ```
pub enum ParameterAny<'a> {
    None,
    ValueInput(value::ParameterValueInput),
    ValueInout(value::ParameterValueInout<'a>),
    ValueOutput(value::ParameterValueOutput<'a>),
    MemrefInput(memref::ParameterMemrefInput<'a>),
    MemrefInout(memref::ParameterMemrefInout<'a>),
    MemrefOutput(memref::ParameterMemrefOutput<'a>),
    /// Unrecognized or implementation-defined type tag.
    ///
    /// Carries the raw `RawParamType` and a mutable reference to the
    /// underlying `TEE_Param` so that the caller can handle it manually.
    Unknown(RawParamType, &'a mut raw::TEE_Param),
}

impl<'a> ParameterAny<'a> {
    pub fn as_value_input(&self) -> Result<&value::ParameterValueInput> {
        match &self {
            Self::ValueInput(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_value_output(&mut self) -> Result<&mut value::ParameterValueOutput<'a>> {
        match self {
            Self::ValueOutput(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_value_inout(&mut self) -> Result<&mut value::ParameterValueInout<'a>> {
        match self {
            Self::ValueInout(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_memref_input(&self) -> Result<&memref::ParameterMemrefInput<'a>> {
        match self {
            Self::MemrefInput(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_memref_inout(&mut self) -> Result<&mut memref::ParameterMemrefInout<'a>> {
        match self {
            Self::MemrefInout(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_memref_output(&mut self) -> Result<&mut memref::ParameterMemrefOutput<'a>> {
        match self {
            Self::MemrefOutput(p) => Ok(p),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
    pub fn as_none(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            _ => Err(ErrorKind::BadParameters.into()),
        }
    }
}

impl<'a> FromRawParameter<'a> for ParameterAny<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut raw::TEE_Param) -> Result<Self> {
        unsafe {
            match raw_type {
                raw::TEE_PARAM_TYPE_NONE => Ok(Self::None),
                raw::TEE_PARAM_TYPE_VALUE_INPUT => Ok(Self::ValueInput(
                    value::ParameterValueInput::from_raw(raw_type, raw_param)?,
                )),
                raw::TEE_PARAM_TYPE_VALUE_INOUT => Ok(Self::ValueInout(
                    value::ParameterValueInout::from_raw(raw_type, raw_param)?,
                )),
                raw::TEE_PARAM_TYPE_VALUE_OUTPUT => Ok(Self::ValueOutput(
                    value::ParameterValueOutput::from_raw(raw_type, raw_param)?,
                )),
                raw::TEE_PARAM_TYPE_MEMREF_INPUT => Ok(Self::MemrefInput(
                    memref::ParameterMemrefInput::from_raw(raw_type, raw_param)?,
                )),
                raw::TEE_PARAM_TYPE_MEMREF_INOUT => Ok(Self::MemrefInout(
                    memref::ParameterMemrefInout::from_raw(raw_type, raw_param)?,
                )),
                raw::TEE_PARAM_TYPE_MEMREF_OUTPUT => Ok(Self::MemrefOutput(
                    memref::ParameterMemrefOutput::from_raw(raw_type, raw_param)?,
                )),
                _ => Ok(Self::Unknown(raw_type, raw_param)),
            }
        }
    }
}

pub type ParametersAny<'a> = (
    ParameterAny<'a>,
    ParameterAny<'a>,
    ParameterAny<'a>,
    ParameterAny<'a>,
);
pub type ParametersNone = (
    none::ParameterNone,
    none::ParameterNone,
    none::ParameterNone,
    none::ParameterNone,
);
