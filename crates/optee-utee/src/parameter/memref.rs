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
//! * [`ParameterMemref`] trait for the shared buffer length.
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
//! Consequently, this module only provides copy-based accessors (`read_to_vec`,
//! `read_at` for reading; `write_at`, `set_output` for writing), mitigating
//! REE-controlled time-of-check-to-time-of-use (TOCTOU) risks. Prefer
//! validating and using the same copy rather than validating one fetch and
//! using another.

use super::{FromRawParameter, ParamType, RawParamType, check_type_is};
use crate::{ErrorKind, Result, raw::TEE_Param};

/// Shared buffer length for a memory-reference parameter.
///
/// Implemented by [`ParameterMemrefInput`], [`ParameterMemrefOutput`],
/// and [`ParameterMemrefInout`]. This is the supertrait of
/// [`ParameterMemrefRead`] and [`ParameterMemrefWrite`].
///
/// The value is the size the host supplied for the shared buffer. OP-TEE
/// validates this range against the caller's shared memory when the TA is
/// invoked, and the accessors in this module bound every read and write by
/// this size, so they cannot access past the host-supplied buffer. For output
/// and in/out parameters the size reported back to the host is updated
/// separately through [`ParameterMemrefWrite::set_updated_size`]; `buffer_len`
/// keeps returning the original host-supplied size.
pub trait ParameterMemref {
    /// Returns the size of the shared buffer as supplied by the host, in bytes.
    fn buffer_len(&self) -> usize;
}

/// Read-only access to a memory-reference parameter's buffer.
///
/// Implemented by [`ParameterMemrefInput`] and [`ParameterMemrefInout`].
///
/// `read_to_vec` returns a TA-owned copy that is unaffected by later REE
/// modification; prefer validating and using the same copy.
pub trait ParameterMemrefRead: ParameterMemref {
    /// Copies the shared buffer into TA-owned memory.
    ///
    /// The allocation is sized from the host-supplied `buffer_len`. Callers
    /// that cannot accept an allocation of that size should bound
    /// `buffer_len` first, or read fixed-size fields with [`Self::read_at`].
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

    /// Copies bytes starting at `offset` into `dest`.
    ///
    /// Returns the number of bytes actually read. The copy is truncated
    /// to the available bytes when `offset + dest.len()` exceeds the
    /// buffer length; reading exactly at the end returns `Ok(0)`.
    ///
    /// Returns `ErrorKind::BadParameters` if `offset` is beyond the end
    /// of the buffer.
    fn read_at(&self, offset: usize, dest: &mut [u8]) -> Result<usize> {
        if offset > self.buffer_len() {
            return Err(ErrorKind::BadParameters.into());
        }
        let available = self.buffer_len() - offset;
        let to_read = core::cmp::min(dest.len(), available);
        if to_read != 0 {
            unsafe {
                crate::raw::TEE_MemMove(
                    dest.as_mut_ptr().cast(),
                    self.buffer_ptr().add(offset).cast(),
                    to_read,
                );
            }
        }
        Ok(to_read)
    }

    /// Returns the start of the shared input buffer.
    #[doc(hidden)]
    fn buffer_ptr(&self) -> *const u8;
}

/// Write access to a memory-reference parameter's buffer.
///
/// Implemented by [`ParameterMemrefOutput`] and [`ParameterMemrefInout`].
///
/// All accessors copy caller-supplied data into shared memory, so callers never
/// hold a direct reference into Normal-World memory.
pub trait ParameterMemrefWrite: ParameterMemref {
    /// Sets the updated size after bounds checking.
    ///
    /// Returns `ErrorKind::ShortBuffer` if `size > buffer_len()`.
    fn set_updated_size(&mut self, size: usize) -> Result<()> {
        if size > self.buffer_len() {
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
    /// the buffer length.
    fn write_at<T: AsRef<[u8]>>(&mut self, offset: usize, data: T) -> Result<()> {
        let input = data.as_ref();
        let new_size = offset
            .checked_add(input.len())
            .ok_or(ErrorKind::ShortBuffer)?;
        if new_size > self.buffer_len() {
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
    /// The `size` must not exceed `buffer_len()`. Prefer
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

impl<'a> ParameterMemref for ParameterMemrefInout<'a> {
    fn buffer_len(&self) -> usize {
        self.capacity
    }
}

impl<'a> ParameterMemref for ParameterMemrefOutput<'a> {
    fn buffer_len(&self) -> usize {
        self.capacity
    }
}

impl<'a> ParameterMemref for ParameterMemrefInput<'a> {
    fn buffer_len(&self) -> usize {
        unsafe { self.0.memref.size }
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefInout<'a> {
    fn buffer_ptr(&mut self) -> *mut u8 {
        unsafe { self.raw_param.memref.buffer as *mut u8 }
    }
    unsafe fn set_updated_size_unchecked(&mut self, size: usize) {
        self.raw_param.memref.size = size;
    }
}

impl<'a> ParameterMemrefWrite for ParameterMemrefOutput<'a> {
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
}

impl<'a> ParameterMemrefRead for ParameterMemrefInput<'a> {
    fn buffer_ptr(&self) -> *const u8 {
        unsafe { self.0.memref.buffer as *const u8 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw;

    fn memref(buffer: *mut u8, size: usize) -> TEE_Param {
        TEE_Param {
            memref: raw::TEE_Param__bindgen_ty_1 {
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

    /// Guarantees the `read_at` boundary contract: reading exactly at the end
    /// returns `Ok(0)` with no shared-memory copy; reading beyond the end
    /// returns `BadParameters` instead of touching out-of-bounds memory; and
    /// offset arithmetic never overflows even with `usize::MAX`.
    #[test]
    fn read_at_boundary_contract() {
        let mut backing = [1u8, 2, 3, 4, 5];
        let mut raw_param = memref(backing.as_mut_ptr(), backing.len());
        let input = unsafe {
            ParameterMemrefInput::from_raw(raw::TEE_PARAM_TYPE_MEMREF_INPUT, &mut raw_param)
        }
        .unwrap();
        // exactly at the end: zero bytes, no copy
        let mut empty: [u8; 0] = [];
        assert_eq!(input.read_at(5, &mut empty).unwrap(), 0);
        let mut too_big = [0u8; 10];
        assert_eq!(input.read_at(5, &mut too_big).unwrap(), 0);
        // beyond the end: caller bug, rejected without copying
        assert_eq!(
            input.read_at(6, &mut too_big).unwrap_err().kind(),
            ErrorKind::BadParameters
        );
        assert_eq!(
            input.read_at(usize::MAX, &mut too_big).unwrap_err().kind(),
            ErrorKind::BadParameters
        );
    }
}
