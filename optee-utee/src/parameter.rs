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

use crate::{
    access::{Accessible, NoAccess, Read, ReadWrite, Writable, Write},
    volatile::VolatileBuf,
    Error, ErrorKind, Result,
};
use core::{
    fmt::{Debug, Display},
    marker,
    ptr::{addr_of_mut, NonNull},
};
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

/// A [`Parameter`] that is a reference to CA-controlled shared memory.
///
/// Note: The memory it points to is volatile and can contain maliciously crafted
/// data, so care should be taken when accessing it. For a safe api, first check
/// the type of the memref via [`Self::in_`], [`Self::out`], or [`Self::inout`].
///
/// These will return type type-safe versions that unsure that read or write access
/// is only allowed based on the underlying [`ParamType`].
///
/// Then, call [`Self::buffer`] which returns a [`VolatileBuf`] that can be accessed
/// safely.
pub struct ParamMemref<'parameter, A = NoAccess> {
    raw: NonNull<raw::Memref>,
    param_type: ParamType,
    capacity: usize,
    _phantom_param: marker::PhantomData<&'parameter mut [u8]>,
    _phantom_access: marker::PhantomData<A>,
}

impl<'parameter, A> ParamMemref<'parameter, A> {
    // Helper function to cast access.
    fn access<NewAccess>(
        self,
        expected_param_type: ParamType,
    ) -> Result<ParamMemref<'parameter, NewAccess>, InvalidAccessErr<Self, NewAccess>> {
        if self.param_type != expected_param_type {
            return Err(InvalidAccessErr {
                param_type: self.param_type,
                value: self,
                _phantom: marker::PhantomData,
            });
        }

        Ok(ParamMemref::<'parameter, NewAccess> {
            raw: self.raw,
            param_type: self.param_type,
            capacity: self.capacity,
            _phantom_param: marker::PhantomData,
            _phantom_access: marker::PhantomData,
        })
    }

    /// Checks that this param is a [`ParamType::MemrefInput`] and casts access to [`Read`].
    pub fn in_(self) -> Result<ParamMemref<'parameter, Read>, InvalidAccessErr<Self, Read>> {
        self.access(ParamType::MemrefInput)
    }

    /// Checks that this param is a [`ParamType::MemrefOutput`] and casts access to [`Write`].
    pub fn out(self) -> Result<ParamMemref<'parameter, Write>, InvalidAccessErr<Self, Write>> {
        self.access(ParamType::MemrefOutput)
    }

    /// Checks that this param is a [`ParamType::MemrefInout`] and casts access to [`ReadWrite`].
    pub fn inout(
        self,
    ) -> Result<ParamMemref<'parameter, ReadWrite>, InvalidAccessErr<Self, ReadWrite>> {
        self.access(ParamType::MemrefInout)
    }

    /// Casts access type to [`NoAccess`], preventing anyone from accessing the buffer
    /// it points to (except by c-style pointer).
    pub fn noaccess(self) -> ParamMemref<'parameter, NoAccess> {
        ParamMemref {
            raw: self.raw,
            param_type: self.param_type,
            capacity: self.capacity,
            _phantom_param: marker::PhantomData,
            _phantom_access: marker::PhantomData,
        }
    }

    pub fn raw(&mut self) -> NonNull<raw::Memref> {
        self.raw
    }

    /// The size of the allocated memory region (i.e. the original value of `self.size`)
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn param_type(&self) -> ParamType {
        self.param_type
    }
}

impl<'parameter, A: Accessible> ParamMemref<'parameter, A> {
    /// Can return `None` if the buffer is null, which is a valid state that can happen
    /// in OP-TEE.
    pub fn buffer(&mut self) -> Option<VolatileBuf<'parameter, A>> {
        let memref = unsafe { self.raw.read() };
        let buf_ptr = memref.buffer.cast::<u8>();
        let buf_ptr = NonNull::new(buf_ptr)?;

        Some(unsafe { VolatileBuf::new(buf_ptr, self.capacity) })
    }
}

impl<'parameter, A: Writable> ParamMemref<'parameter, A> {
    /// Errors if the new size would be bigger than `self.capacity()`
    pub fn set_updated_size(&mut self, size: usize) -> Result<(), BiggerThanCapacityErr> {
        if size > self.capacity {
            return Err(BiggerThanCapacityErr {
                requested_size: size,
                capacity: self.capacity,
            });
        }
        let memref = unsafe { self.raw.as_mut() };
        memref.size = size;

        Ok(())
    }

    /// By convention, setting the raw size to something larger than what was initially passed by OP-TEE, is a signal to
    /// the CA that there is insufficient memory allocated to output a result, and that the CA should invoke the function again
    /// with a larger buffer. See section `4.3.6.4` of the Global Platform TEE Internal Core API.
    /// Note that this does *not* update `self.capacity()`.
    /// Returns an error if the requested amount is not any bigger than the existing capacity.
    pub fn request_more_capacity(
        &mut self,
        new_capacity: usize,
    ) -> Result<(), NotBiggerThanCapacityErr> {
        if new_capacity <= self.capacity {
            return Err(NotBiggerThanCapacityErr {
                requested_capacity: new_capacity,
                capacity: self.capacity,
            });
        }
        let memref = unsafe { self.raw.as_mut() };
        memref.size = new_capacity;

        Ok(())
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
                    raw: NonNull::new_unchecked(addr_of_mut!((*self.raw).memref)),
                    param_type: self.param_type,
                    capacity: unsafe { *self.raw }.memref.size,
                    _phantom_access: marker::PhantomData,
                    _phantom_param: marker::PhantomData,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ParamType {
    None = 0,
    ValueInput = 1,
    ValueOutput = 2,
    ValueInout = 3,
    MemrefInput = 5,
    MemrefOutput = 6,
    MemrefInout = 7,
}

impl Display for ParamType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParamType::None => write!(f, "None"),
            ParamType::ValueInput => write!(f, "ValueInput"),
            ParamType::ValueOutput => write!(f, "ValueOutput"),
            ParamType::ValueInout => write!(f, "ValueInout"),
            ParamType::MemrefInput => write!(f, "MemrefInput"),
            ParamType::MemrefOutput => write!(f, "MemrefOutput"),
            ParamType::MemrefInout => write!(f, "MemrefInout"),
        }
    }
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

#[derive(Debug)]
pub struct BiggerThanCapacityErr {
    requested_size: usize,
    capacity: usize,
}

impl Display for BiggerThanCapacityErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "requested size {} but this is bigger than the capacity {}",
            self.requested_size, self.capacity,
        )
    }
}

impl core::error::Error for BiggerThanCapacityErr {}

#[derive(Debug)]
pub struct NotBiggerThanCapacityErr {
    requested_capacity: usize,
    capacity: usize,
}

impl Display for NotBiggerThanCapacityErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "requested capacity {} but should have been bigger the original capacity {}",
            self.requested_capacity, self.capacity,
        )
    }
}

impl core::error::Error for NotBiggerThanCapacityErr {}

/// Error while attempting to cast access
#[derive(Debug, Clone)]
pub struct InvalidAccessErr<T, NewAccess> {
    param_type: ParamType,
    value: T,
    _phantom: marker::PhantomData<NewAccess>,
}

impl<T, NewAccess> Display for InvalidAccessErr<T, NewAccess> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "unable to cast access to {}, param_type was {}",
            core::any::type_name::<NewAccess>(),
            self.param_type
        )
    }
}

impl<T: Debug + Display, NewAccess: Debug + Display> core::error::Error
    for InvalidAccessErr<T, NewAccess>
{
}

impl<T, NewAccess> InvalidAccessErr<T, NewAccess> {
    pub fn into_inner(self) -> T {
        self.value
    }
}
