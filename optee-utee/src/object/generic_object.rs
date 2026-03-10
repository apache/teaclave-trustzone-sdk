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

use core::mem;

use super::{AttributeId, ObjectInfo, UsageFlag};
use crate::{Error, Result};

use optee_utee_sys as raw;

/// A generic trait for an object (transient or persistent).
pub trait GenericObject {
    /// Return the handle of an object.
    fn handle(&self) -> raw::TEE_ObjectHandle;

    /// Return the characteristics of an object.
    ///
    /// # Errors
    ///
    /// For [PersistentObject](crate::PersistentObject):
    ///
    /// * `CorruptObject`: If the persistent object is corrupt. The object
    ///   handle SHALL behave based on the
    ///   `gpd.ta.doesNotCloseHandleOnCorruptObject` property.
    /// * `StorageNotAvailable`: If the persistent object is stored in a
    ///   storage area which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// * If object is not a valid opened object handle.
    /// * If the implementation detects any other error associated with this
    ///   function that is not explicitly associated with a defined return code
    ///   for this function.
    fn info(&self) -> Result<ObjectInfo> {
        let mut raw_info: raw::TEE_ObjectInfo = unsafe { mem::zeroed() };
        match unsafe { raw::TEE_GetObjectInfo1(self.handle(), &mut raw_info) } {
            raw::TEE_SUCCESS => Ok(ObjectInfo::from_raw(raw_info)),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Restrict the object usage flags of an object handle to contain at most
    /// the flags passed in the obj_usage parameter.
    ///
    /// # Errors
    ///
    /// For [PersistentObject](crate::PersistentObject):
    ///
    /// * `CorruptObject`: If the persistent object is corrupt. The object
    ///   handle SHALL behave based on the
    ///   `gpd.ta.doesNotCloseHandleOnCorruptObject` property.
    /// * `StorageNotAvailable`: If the persistent object is stored in a
    ///   storage area which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// * If object is not a valid opened object handle.
    /// * If the implementation detects any other error associated with this
    ///   function that is not explicitly associated with a defined return code
    ///   for this function.
    fn restrict_usage(&mut self, obj_usage: UsageFlag) -> Result<()> {
        match unsafe { raw::TEE_RestrictObjectUsage1(self.handle(), obj_usage.bits()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Extract one buffer attribute from an object. The attribute is
    /// identified by the argument id.
    ///
    /// # Errors
    ///
    /// For all:
    ///
    /// * `ItemNotFound`: If the attribute is not found on this object.
    /// * `SHORT_BUFFER`: If buffer is NULL or too small to contain the key
    ///   part.
    ///
    /// For [PersistentObject](crate::PersistentObject):
    ///
    /// * `CorruptObject`: If the persistent object is corrupt. The object
    ///   handle SHALL behave based on the
    ///   `gpd.ta.doesNotCloseHandleOnCorruptObject` property.
    /// * `StorageNotAvailable`: If the persistent object is stored in a
    ///   storage area which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// * If object is not a valid opened object handle.
    /// * If the object is not initialized.
    /// * If Bit [29] of attributeID is not set to 0, so the attribute is not a
    ///   buffer attribute.
    /// * If Bit [28] of attributeID is set to 0, denoting a protected
    ///   attribute, and the object usage does not contain the
    ///   TEE_USAGE_EXTRACTABLE flag.
    /// * If the implementation detects any other error associated with this
    ///   function that is not explicitly associated with a defined return code
    ///   for this function.
    fn ref_attribute(&self, id: AttributeId, buffer: &mut [u8]) -> Result<usize> {
        let mut size = buffer.len();
        match unsafe {
            raw::TEE_GetObjectBufferAttribute(
                self.handle(),
                id as u32,
                buffer as *mut _ as _,
                &mut size,
            )
        } {
            raw::TEE_SUCCESS => Ok(size),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Extract one value attribute from an object. The attribute is identified
    /// by the argument id.
    ///
    /// # Errors
    ///
    /// For all:
    ///
    /// * `ItemNotFound`: If the attribute is not found on this object.
    /// * `SHORT_BUFFER`: If buffer is NULL or too small to contain the key
    ///   part.
    ///
    /// For [PersistentObject](crate::PersistentObject):
    ///
    /// * `CorruptObject`: If the persistent object is corrupt. The object
    ///   handle SHALL behave based on the
    ///   `gpd.ta.doesNotCloseHandleOnCorruptObject` property.
    /// * `StorageNotAvailable`: If the persistent object is stored in a
    ///   storage area which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// * If object is not a valid opened object handle.
    /// * If the object is not initialized.
    /// * If Bit [29] of attributeID is not set to 0, so the attribute is not a
    ///   buffer attribute.
    /// * If Bit [28] of attributeID is set to 0, denoting a protected
    ///   attribute, and the object usage does not contain the
    ///   TEE_USAGE_EXTRACTABLE flag.
    /// * If the implementation detects any other error associated with this
    ///   function that is not explicitly associated with a defined return code
    ///   for this function.
    fn value_attribute(&self, id: u32) -> Result<(u32, u32)> {
        let mut value_a: u32 = 0;
        let mut value_b: u32 = 0;
        match unsafe {
            raw::TEE_GetObjectValueAttribute(
                self.handle(),
                id,
                &mut value_a as *mut _,
                &mut value_b as *mut _,
            )
        } {
            raw::TEE_SUCCESS => Ok((value_a, value_b)),
            code => Err(Error::from_raw_error(code)),
        }
    }
}
