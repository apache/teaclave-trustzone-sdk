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
//!
//! # Shared-memory safety
//!
//! The buffers are mapped from Normal World, which may access them concurrently.
//! Consequently, `get_buffer` and `get_buffer_mut` are unsafe: callers may use
//! them only when their application guarantees that the REE will not access the
//! memory for the returned reference's lifetime. Callers are also responsible
//! for preventing REE-controlled time-of-check-to-time-of-use (TOCTOU) attacks;
//! data must not be validated through a shared slice and then fetched from it
//! again for use. Use `read_to_vec` and validate/use the resulting TA-owned copy,
//! or use `write_at`/`set_output`, when those guarantees are unavailable.

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
    /// # Safety
    ///
    /// This slice points directly into Normal-World shared memory. The caller
    /// must ensure the REE cannot mutate that memory for the entire lifetime of
    /// the returned slice. The caller must also prevent TOCTOU attacks: do not
    /// validate data through this slice and later fetch it again from shared
    /// memory for use. If either guarantee cannot be met, use
    /// [`Self::read_to_vec`] and perform both validation and use on the same
    /// TA-owned copy.
    unsafe fn get_buffer(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.buffer_ptr(), self.buffer_len()) }
    }

    /// Copies the shared buffer into TA-owned memory.
    fn read_to_vec(&self) -> alloc::vec::Vec<u8> {
        let len = self.buffer_len();
        let mut copy = alloc::vec![0; len];
        if len != 0 {
            unsafe {
                crate::raw::TEE_MemMove(copy.as_mut_ptr().cast(), self.buffer_ptr().cast(), len);
            }
        }
        copy
    }

    /// Returns the start of the shared input buffer.
    #[doc(hidden)]
    fn buffer_ptr(&self) -> *const u8;

    /// Returns the length of the shared input buffer.
    #[doc(hidden)]
    fn buffer_len(&self) -> usize;
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
    /// # Safety
    ///
    /// This slice points directly into Normal-World shared memory. The caller
    /// must ensure the REE does not read or write that memory for the entire
    /// lifetime of the returned mutable slice. A TA that only writes output
    /// need not protect the resulting contents from the REE. However, if the TA
    /// also reads, validates, or makes decisions from this slice, it must treat
    /// those bytes like input shared memory and prevent TOCTOU attacks. Prefer
    /// [`Self::write_at`] for write-only access; copy data into TA-owned memory
    /// before validating or otherwise relying on bytes read from this slice.
    unsafe fn get_buffer_mut(&mut self) -> &mut [u8] {
        let capacity = self.get_capacity();
        unsafe { core::slice::from_raw_parts_mut(self.buffer_ptr(), capacity) }
    }

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
        let new_size = offset
            .checked_add(input.len())
            .ok_or(ErrorKind::ShortBuffer)?;
        if new_size > self.get_capacity() {
            return Err(ErrorKind::ShortBuffer.into());
        }
        if !input.is_empty() {
            unsafe {
                crate::raw::TEE_MemMove(
                    self.buffer_ptr().add(offset).cast(),
                    input.as_ptr().cast(),
                    input.len(),
                );
            }
        }
        unsafe { self.set_updated_size_unchecked(new_size) };
        Ok(())
    }

    /// Returns the start of the shared output buffer.
    #[doc(hidden)]
    fn buffer_ptr(&mut self) -> *mut u8;

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
        if unsafe { raw_param.memref.buffer }.is_null() {
            return Err(ErrorKind::BadParameters.into());
        }
        Ok(Self(raw_param))
    }
}
impl<'a> FromRawParameter<'a> for ParameterMemrefInout<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::MemrefInout)?;
        if unsafe { raw_param.memref.buffer }.is_null() {
            return Err(ErrorKind::BadParameters.into());
        }
        Ok(Self {
            capacity: unsafe { raw_param.memref.size },
            raw_param,
        })
    }
}
impl<'a> FromRawParameter<'a> for ParameterMemrefOutput<'a> {
    unsafe fn from_raw(raw_type: RawParamType, raw_param: &'a mut TEE_Param) -> Result<Self> {
        check_type_is(raw_type, ParamType::MemrefOutput)?;
        if unsafe { raw_param.memref.buffer }.is_null() {
            return Err(ErrorKind::BadParameters.into());
        }
        Ok(Self {
            capacity: unsafe { raw_param.memref.size },
            raw_param,
        })
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefInout<'a> {
    fn get_capacity(&self) -> usize {
        self.capacity
    }
    fn buffer_ptr(&mut self) -> *mut u8 {
        unsafe { self.raw_param.memref.buffer as *mut u8 }
    }
    unsafe fn set_updated_size_unchecked(&mut self, size: usize) {
        self.raw_param.memref.size = size;
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefOutput<'a> {
    fn get_capacity(&self) -> usize {
        self.capacity
    }
    fn buffer_ptr(&mut self) -> *mut u8 {
        unsafe { self.raw_param.memref.buffer as *mut u8 }
    }
    unsafe fn set_updated_size_unchecked(&mut self, size: usize) {
        self.raw_param.memref.size = size;
    }
}

impl<'a> ParameterMemrefRead for ParameterMemrefInout<'a> {
    fn buffer_ptr(&self) -> *const u8 {
        unsafe { self.raw_param.memref.buffer as *const u8 }
    }
    fn buffer_len(&self) -> usize {
        self.capacity
    }
}

impl<'a> ParameterMemrefRead for ParameterMemrefInput<'a> {
    fn buffer_ptr(&self) -> *const u8 {
        unsafe { self.0.memref.buffer as *const u8 }
    }
    fn buffer_len(&self) -> usize {
        unsafe { self.0.memref.size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw;

    fn memref(buffer: *mut u8, size: usize) -> TEE_Param {
        TEE_Param {
            memref: raw::Memref {
                buffer: buffer.cast(),
                size,
            },
        }
    }

    #[test]
    fn typed_wrappers_reject_null_buffers() {
        let cases = [
            (raw::TEE_PARAM_TYPE_MEMREF_INPUT, ParamType::MemrefInput),
            (raw::TEE_PARAM_TYPE_MEMREF_INOUT, ParamType::MemrefInout),
            (raw::TEE_PARAM_TYPE_MEMREF_OUTPUT, ParamType::MemrefOutput),
        ];

        for (raw_type, param_type) in cases {
            let mut raw_param = memref(core::ptr::null_mut(), 0);
            let error = match param_type {
                ParamType::MemrefInput => unsafe {
                    ParameterMemrefInput::from_raw(raw_type, &mut raw_param).map(|_| ())
                },
                ParamType::MemrefInout => unsafe {
                    ParameterMemrefInout::from_raw(raw_type, &mut raw_param).map(|_| ())
                },
                ParamType::MemrefOutput => unsafe {
                    ParameterMemrefOutput::from_raw(raw_type, &mut raw_param).map(|_| ())
                },
                _ => unreachable!(),
            }
            .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::BadParameters);
        }
    }

    #[test]
    fn zero_length_non_null_buffer_is_valid() {
        let mut raw_param = memref(core::ptr::NonNull::<u8>::dangling().as_ptr(), 0);
        let input = unsafe {
            ParameterMemrefInput::from_raw(raw::TEE_PARAM_TYPE_MEMREF_INPUT, &mut raw_param)
        }
        .unwrap();
        assert!(input.read_to_vec().is_empty());
    }
}
