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

use alloc::boxed::Box;

use optee_utee_sys as raw;

use super::ObjectInfo;
use crate::{Error, Result};

// TODO: The examples and detailed function explanation will be added after we
// test this struct and its functions.
/// An enumerator for [PersistentObject](crate::PersistentObject)s.
pub struct ObjectEnumHandle {
    raw: *mut raw::TEE_ObjectEnumHandle,
}

impl ObjectEnumHandle {
    /// Allocate an object enumerator.
    /// Once an object enumerator has been allocated, it can be reused for
    /// multiple enumerations.
    pub fn allocate() -> Result<Self> {
        let raw_handle: *mut raw::TEE_ObjectEnumHandle =
            Box::into_raw(Box::new(core::ptr::null_mut()));
        match unsafe { raw::TEE_AllocatePersistentObjectEnumerator(raw_handle) } {
            raw::TEE_SUCCESS => Ok(Self { raw: raw_handle }),
            code => {
                unsafe {
                    drop(Box::from_raw(raw_handle));
                }
                Err(Error::from_raw_error(code))
            }
        }
    }

    /// Reset an object enumerator handle to its initial state after allocation.
    /// If an enumeration has been started, it is stopped.
    pub fn reset(&mut self) {
        unsafe {
            raw::TEE_ResetPersistentObjectEnumerator(*self.raw);
        }
    }

    /// Start the enumeration of all the
    /// [PersistentObject](crate::PersistentObject)s in a given Trusted Storage.
    /// The object information can be retrieved by calling the function
    /// [ObjectEnumHandle::get_next](crate::ObjectEnumHandle::get_next)
    /// repeatedly.
    pub fn start(&mut self, storage_id: u32) -> Result<()> {
        match unsafe { raw::TEE_StartPersistentObjectEnumerator(*self.raw, storage_id) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Get the next object in an enumeration and returns information about the
    /// object: type, size, identifier, etc.
    pub fn get_next<T>(
        &mut self,
        object_info: &mut ObjectInfo,
        object_id: &mut [u8],
    ) -> Result<u32> {
        let mut object_id_len: usize = 0;
        match unsafe {
            raw::TEE_GetNextPersistentObject(
                *self.raw,
                &mut object_info.raw,
                object_id.as_mut_ptr() as _,
                &mut object_id_len,
            )
        } {
            raw::TEE_SUCCESS => Ok(object_id_len as u32),
            code => Err(Error::from_raw_error(code)),
        }
    }
}

impl Drop for ObjectEnumHandle {
    /// Deallocates all resources associated with an object enumerator handle.
    /// After this function is called, the handle is no longer valid.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid opened object.
    /// 2) If the Implementation detects any other error.
    fn drop(&mut self) {
        unsafe {
            raw::TEE_FreePersistentObjectEnumerator(*self.raw);
            drop(Box::from_raw(self.raw));
        }
    }
}
