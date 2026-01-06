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

use optee_utee_sys as raw;

use super::GenericObject;

/// An opaque handle on an object.
#[derive(Debug)]
pub struct ObjectHandle(raw::TEE_ObjectHandle);

impl ObjectHandle {
    pub fn from_raw(raw: raw::TEE_ObjectHandle) -> crate::Result<ObjectHandle> {
        if raw.is_null() {
            return Err(crate::ErrorKind::BadParameters.into());
        }
        Ok(Self(raw))
    }

    pub fn handle(&self) -> raw::TEE_ObjectHandle {
        self.0
    }

    /// Forget the inner handle to prevent a double-free, this function would be
    /// called when the inner handle is(or will be) freed externally.
    ///
    /// Example:
    /// ``` rust,no_run
    /// # use optee_utee::ObjectHandle;
    /// # use optee_utee_sys as raw;
    /// # let external_handle: raw::TEE_ObjectHandle = core::ptr::null_mut();
    /// # fn main() -> optee_utee::Result<()> {
    /// # let external_handle = core::ptr::null_mut();
    /// // `external_handle` is a handle that is constructed and controlled
    /// // externally.
    /// // `handle` is valid, and will call TEE_CloseObject on
    /// // `external_handle` when it is dropping, which is not allowed
    /// // as the `external_handle` is externally controlled.
    /// let mut handle = ObjectHandle::from_raw(external_handle)?;
    /// // ... Some operation
    /// // forget the inner handle, so it won't call TEE_CloseObject on
    /// // `external_handle`
    /// handle.forget();
    /// # Ok(())
    /// # }
    /// ```
    pub fn forget(mut self) {
        self.0 = core::ptr::null_mut();
    }
}

// functions for internal usage
impl ObjectHandle {
    pub(crate) fn new_null() -> Self {
        Self(core::ptr::null_mut())
    }

    pub(crate) fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl Drop for ObjectHandle {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe { raw::TEE_CloseObject(self.handle()) }
        }
    }
}

impl GenericObject for ObjectHandle {
    fn handle(&self) -> raw::TEE_ObjectHandle {
        self.handle()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use optee_utee_sys::{
        mock_api,
        mock_utils::{object::MockHandle, SERIAL_TEST_LOCK},
    };

    use super::*;

    /// Ensures `ObjectHandle` can be safely constructed from a raw handle
    /// and automatically calls `TEE_CloseObject` when dropped.
    #[test]
    fn test_from_raw() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();
        let fn1 = mock_api::TEE_CloseObject_context();

        fn1.expect().return_once_st(move |obj| {
            debug_assert_eq!(obj, handle.clone());
        });

        let obj = ObjectHandle::from_raw(handle.clone()).expect("it should be ok");
        assert_eq!(obj.handle(), handle);
    }

    /// Ensures `ObjectHandle` can call `forget` to prevent automatically
    /// calls `TEE_CloseObject` when dropped.
    #[test]
    fn test_forget() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();

        let obj = ObjectHandle::from_raw(handle.clone()).expect("it should be ok");
        assert_eq!(obj.handle(), handle);

        obj.forget();
    }

    #[test]
    fn test_new_null() {
        let obj = ObjectHandle::new_null();
        assert!(obj.is_null());
    }
}
