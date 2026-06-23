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

//! Type-safe wrappers for TEE memory-reference parameters.
//!
//! A *memref* (memory reference) parameter maps a shared-memory buffer
//! between the host and the TA. Unlike value parameters which carry two
//! `u32` values, memrefs can transport arbitrary byte sequences.
//!
//! This module provides:
//!
//! * [`ParameterMemrefRead`] trait for reading the buffer contents.
//! * [`ParameterMemrefWrite`] trait for writing into buffers and
//!   reporting updated sizes.
//! * Three concrete wrappers encoding the data direction:
//!   [`ParameterMemrefInput`], [`ParameterMemrefOutput`],
//!   [`ParameterMemrefInout`].
//!
//! # Direction guarantees
//!
//! | Type | Host → TA | TA → Host |
//! |---|---|---|
//! | `ParameterMemrefInput` | ✓ | ✗ |
//! | `ParameterMemrefOutput` | ✗ | ✓ |
//! | `ParameterMemrefInout` | ✓ | ✓ |

use super::{FromRawParameter, ParamType, RawParamType, check_type_is};
use crate::{ErrorKind, Result, raw::TEE_Param};

/// Read-only access to a memory-reference parameter's buffer.
///
/// Implemented by [`ParameterMemrefInput`] and [`ParameterMemrefInout`].
pub trait ParameterMemrefRead {
    /// Returns the buffer contents as a byte slice.
    ///
    /// For `ParameterMemrefInput` the length is the original buffer size as
    /// supplied by the host. For `ParameterMemrefInout` the length is the
    /// full buffer capacity, not the number of valid bytes (which may have
    /// been updated by a prior write).
    fn get_buffer(&self) -> &[u8];
}

/// Write access to a memory-reference parameter's buffer.
///
/// Implemented by [`ParameterMemrefOutput`] and [`ParameterMemrefInout`].
pub trait ParameterMemrefWrite {
    /// Returns a mutable byte slice representing the output buffer.
    ///
    /// After writing to the returned buffer, call
    /// [`ParameterMemrefWrite::set_updated_size`] to report how many bytes were
    /// produced. Otherwise the client application may observe an incorrect
    /// output size.
    fn get_buffer_mut(&mut self) -> &mut [u8];

    /// Returns the maximum allowed buffer size (capacity).
    fn get_capacity(&self) -> usize;

    /// Sets the updated size after bounds checking.
    ///
    /// Returns `ErrorKind::ShortBuffer` if `size > get_capacity()`.
    fn set_updated_size(&mut self, size: usize) -> Result<()> {
        if size > self.get_capacity() {
            return Err(ErrorKind::ShortBuffer.into());
        }
        unsafe { self.set_updated_size_unchecked(size) };
        Ok(())
    }

    /// Copies `data` into the buffer, then updates the reported size.
    fn set_output<T: AsRef<[u8]>>(&mut self, data: T) -> Result<()> {
        self.write_at(0, data)
    }

    /// Copies `data` into the buffer at the given `offset`, then updates the
    /// reported size to `offset + data.len()`.
    ///
    /// Returns `ErrorKind::ShortBuffer` if the new size would exceed
    /// the buffer capacity.
    fn write_at<T: AsRef<[u8]>>(&mut self, offset: usize, data: T) -> Result<()> {
        let input = data.as_ref();
        let new_size = offset + input.len();
        if new_size > self.get_capacity() {
            return Err(ErrorKind::ShortBuffer.into());
        }
        let output = self.get_buffer_mut();
        output[offset..new_size].copy_from_slice(input);
        unsafe { self.set_updated_size_unchecked(new_size) };
        Ok(())
    }

    /// Directly sets the updated size without bounds checking.
    ///
    /// # Safety
    ///
    /// The `size` must not exceed `get_capacity()`. Prefer
    /// [`ParameterMemrefWrite::set_updated_size`] unless the caller has already
    /// checked the bounds.
    unsafe fn set_updated_size_unchecked(&mut self, size: usize);
}

/// A memory-reference input parameter.
///
/// The host passes a read-only buffer to the TA. The length is the
/// original buffer size as specified by the host.
pub struct ParameterMemrefInput<'a>(&'a TEE_Param);

/// A memory-reference in/out parameter.
///
/// The host passes a read-write buffer. The TA may read the initial contents,
/// overwrite them, and report the final number of valid bytes.
pub struct ParameterMemrefInout<'a> {
    capacity: usize,
    raw_param: &'a mut TEE_Param,
}

/// A memory-reference output parameter.
///
/// The host provides a write-only buffer. The TA fills the buffer and report
/// the final number of valid bytes via
/// [`ParameterMemrefWrite::set_updated_size`].
pub struct ParameterMemrefOutput<'a> {
    capacity: usize,
    raw_param: &'a mut TEE_Param,
}

impl<'a> FromRawParameter<'a> for ParameterMemrefInput<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::MemrefInput)?;
        Ok(Self(raw_param))
    }
}
impl<'a> FromRawParameter<'a> for ParameterMemrefInout<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::MemrefInout)?;
        Ok(Self {
            capacity: unsafe { raw_param.memref.size },
            raw_param,
        })
    }
}
impl<'a> FromRawParameter<'a> for ParameterMemrefOutput<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::MemrefOutput)?;
        Ok(Self {
            capacity: unsafe { raw_param.memref.size },
            raw_param,
        })
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefInout<'a> {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.raw_param.memref.buffer as *mut u8, self.capacity)
        }
    }
    fn get_capacity(&self) -> usize {
        self.capacity
    }
    unsafe fn set_updated_size_unchecked(&mut self, size: usize) {
        self.raw_param.memref.size = size;
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefOutput<'a> {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.raw_param.memref.buffer as *mut u8, self.capacity)
        }
    }
    fn get_capacity(&self) -> usize {
        self.capacity
    }
    unsafe fn set_updated_size_unchecked(&mut self, size: usize) {
        self.raw_param.memref.size = size;
    }
}

impl<'a> ParameterMemrefRead for ParameterMemrefInout<'a> {
    fn get_buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.raw_param.memref.buffer as *const u8, self.capacity)
        }
    }
}

impl<'a> ParameterMemrefRead for ParameterMemrefInput<'a> {
    fn get_buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.0.memref.buffer as *const u8, self.0.memref.size)
        }
    }
}
