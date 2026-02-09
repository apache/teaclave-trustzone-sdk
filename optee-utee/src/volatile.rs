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

//! Provides safe ways to interact with volatile buffers
//!
//! # A note about reading from untrusted volatile memory
//! Reading from a `VolatileBuf` requires *both* that `A` is [`Readable`] and that `T`
//! implements [`bytemuck::Pod`].
//!
//! Because the buffer typically lives in CA controlled shared memory, we need to be
//! careful when reading this memory because we have zero guarantees about the
//! bit-level representation of the data being read. A malicious CA could, for example,
//! try to put non-utf8 bytes into a `str`, or populate an enum variant with an invalid
//! value.
//!
//! To guard against malicious bit-level manipulation by a CA, we require that *any*
//! valid bit pattern is a valid `T`. This is made possible via the [`bytemuck::Pod`]
//! trait. This guarantees that such reads are safe, even in the presence of malicous
//! bit fiddling.
//!
//! Luckily most types implement this trait, including `u8` and other such primitives,
//! and bytemuck provides derive macros for implementing it on your own first-party
//! types if you have the need for that.

use core::{fmt::Display, marker::PhantomData, ptr::NonNull};

use crate::{
    access::{NoAccess, Readable, Writable},
    error::CoreError,
    ErrorKind,
};

/// A volatile buffer of memory. Unlike regular slices, volatile buffers cannot ever
/// safely have references constructed. To circumvent this limitation, only copying
/// to/from the buffer (or its indicies) is supported.
///
/// Note that due to the nature of volatile memory, reading / writing to the same
/// indices does not guarantee that subsequent reads/writes will have the previous
/// value, even with an `&mut VolatileBuf`.
#[derive(Debug)]
pub struct VolatileBuf<'parameter, A, T = u8> {
    ptr: NonNull<T>,
    capacity: usize,
    _phantom_param: PhantomData<&'parameter mut [T]>,
    _phantom_access: PhantomData<A>,
}

impl<'parameter, A, T> VolatileBuf<'parameter, A, T> {
    /// We make this pub(crate) to limit use of this api, since its rather hard to get
    /// the invariants right and user code should not muck about with those invariants.
    ///
    /// # SAFETY
    /// `ptr` must be:
    /// * a contiguous memory region of length `capacity` items or less
    /// * if `A` is [`Writable`], then `ptr` must uphold the same safety
    ///   requirements as [`core::ptr::write_volatile`].
    /// * if `A` is [`Readable`], then `ptr` must uphold the same safety
    ///   requirements as [`core::ptr::read_volatile`].
    pub(crate) unsafe fn new(ptr: NonNull<T>, capacity: usize) -> Self {
        Self {
            ptr,
            capacity,
            _phantom_param: PhantomData,
            _phantom_access: PhantomData,
        }
    }

    /// The maximum number of elements in the buffer
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Gets access to the underlying raw pointer. This is safe, because dereferencing
    /// the raw pointer requires `unsafe`.
    #[inline]
    pub fn raw(&self) -> NonNull<T> {
        self.ptr
    }
}

impl<'parameter, T> VolatileBuf<'parameter, NoAccess, T> {
    /// Unlike [`Self::new`], this is always safe since it is impossible to access
    /// the underlying buffer.
    pub fn new_no_access(ptr: NonNull<T>, capacity: usize) -> Self {
        Self {
            ptr,
            capacity,
            _phantom_param: PhantomData,
            _phantom_access: PhantomData,
        }
    }
}

impl<'parameter, A: Readable, T: bytemuck::Pod> VolatileBuf<'parameter, A, T> {
    /// Copies the buffer to `out`.
    ///
    /// Returns `WouldOverflowErr` if the output buffer is smaller than
    /// `self.capacity()`.
    pub fn copy_to(&self, out: &mut [T]) -> Result<(), InsufficientBufferSizeErr> {
        if out.len() < self.capacity {
            return Err(InsufficientBufferSizeErr);
        }

        for (idx, v) in out.iter_mut().enumerate() {
            // SAFETY: Because `T` is a POD type, and because OP-TEE guarantees that an
            // In/Inout param is readable across its entire buffer, we know that no
            // matter what the CA does to the memory, the resulting bit representation
            // is valid. Because `T` is Readable, its also known that reading from this
            // is OK (and the caller of `Self::new` has been informed to uphold this
            // invariant).
            *v = unsafe { core::ptr::read_volatile(self.ptr.add(idx).as_ptr()) };
        }

        Ok(())
    }

    /// Reads the value at the given index.
    ///
    /// Returns [`OutOfBoundsErr`] if `index >= self.capacity()`.
    pub fn read(&self, index: usize) -> Result<T, OutOfBoundsErr> {
        if index >= self.capacity {
            return Err(OutOfBoundsErr);
        }

        // SAFETY: Because `T` is a POD type, and because OP-TEE guarantees that an
        // In/Inout param is readable across its entire buffer, we know that no matter
        // what the CA does to the memory, the resulting bit representation is valid.
        // Because `T` is Readable, its also known that reading from this is OK (and
        // the caller of `Self::new` has been informed to uphold this invariant).
        Ok(unsafe { core::ptr::read_volatile(self.ptr.add(index).as_ptr()) })
    }
}

impl<'parameter, A: Writable, T: Copy> VolatileBuf<'parameter, A, T> {
    /// Copies all values from `values` into `self`.
    ///
    /// Returns [`InsufficientBufferSizeErr`] if the copy would exceed the capacity
    /// of `self`.
    pub fn copy_from(&mut self, values: &[T]) -> Result<(), InsufficientBufferSizeErr> {
        if values.len() > self.capacity {
            return Err(InsufficientBufferSizeErr);
        }

        for (idx, v) in values.iter().copied().enumerate() {
            unsafe { core::ptr::write_volatile(self.ptr.add(idx).as_ptr(), v) };
        }

        Ok(())
    }

    /// Writes to a location in the buffer.
    ///
    /// Returns [`OutOfBoundsErr`] if `index >= self.capacity()`.
    pub fn write(&mut self, index: usize, value: T) -> Result<(), OutOfBoundsErr> {
        if index >= self.capacity {
            return Err(OutOfBoundsErr);
        }

        unsafe { core::ptr::write_volatile(self.ptr.add(index).as_ptr(), value) };

        Ok(())
    }
}

/// Copying would fail due to an insufficiently sized buffer.
#[derive(Debug)]
pub struct InsufficientBufferSizeErr;

impl Display for InsufficientBufferSizeErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "copy would fail due to insufficiently sized buffer")
    }
}

impl CoreError for InsufficientBufferSizeErr {}

impl From<InsufficientBufferSizeErr> for crate::Error {
    fn from(_value: InsufficientBufferSizeErr) -> Self {
        crate::Error::new(ErrorKind::ShortBuffer)
    }
}

#[derive(Debug, Clone)]
pub struct OutOfBoundsErr;

impl Display for OutOfBoundsErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "index out of bounds")
    }
}

impl CoreError for OutOfBoundsErr {}

impl From<OutOfBoundsErr> for crate::Error {
    fn from(_value: OutOfBoundsErr) -> Self {
        crate::Error::new(ErrorKind::Overflow)
    }
}
