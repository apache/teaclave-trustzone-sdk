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

use super::{DataFlag, GenericObject, ObjectHandle, ObjectStorageConstants, Whence};
use crate::{Error, Result};

/// An object identified by an Object Identifier and including a Data Stream.
///
/// Contrast [TransientObject](crate::TransientObject).
#[derive(Debug)]
pub struct PersistentObject(ObjectHandle);

impl PersistentObject {
    /// Open an existing persistent object.
    ///
    /// # Parameters
    ///
    /// 1) `storage_id`: The storage to use which is defined in
    ///    [ObjectStorageConstants](crate::ObjectStorageConstants).
    /// 2) `object_id`: The object identifier. Note that this buffer cannot
    ///    reside in shared memory.
    /// 3) `flags`: The [DataFlag](crate::DataFlag) which determine the settings
    ///    under which the object is opened.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open(
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_READ) {
    ///     Ok(object) =>
    ///     {
    ///         // ...
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `ItemNotFound`: If the storage denoted by storage_id does not exist
    ///    or if the object identifier cannot be found in the storage.
    /// 2) `Access_Conflict`: If an access right conflict was detected while
    ///    opening the object.
    /// 3) `OutOfMemory`: If there is not enough memory to complete the
    ///    operation.
    /// 4) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 5) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object_id.len() >
    ///    [MiscellaneousConstants::TeeObjectIdMaxLen](crate::MiscellaneousConstants::TeeObjectIdMaxLen)
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn open(
        storage_id: ObjectStorageConstants,
        object_id: &[u8],
        flags: DataFlag,
    ) -> Result<Self> {
        let mut handle: raw::TEE_ObjectHandle = core::ptr::null_mut();
        // Move as much code as possible out of unsafe blocks to maximize Rust’s
        // safety checks.
        let handle_mut = &mut handle;
        match unsafe {
            raw::TEE_OpenPersistentObject(
                storage_id as u32,
                object_id.as_ptr() as _,
                object_id.len(),
                flags.bits(),
                handle_mut,
            )
        } {
            raw::TEE_SUCCESS => Ok(Self(ObjectHandle::from_raw(handle)?)),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create an object with initial attributes and an initial data stream
    /// content.
    ///
    /// # Parameters
    ///
    /// 1) `storage_id`: The storage to use which is defined in
    ///    [ObjectStorageConstants](crate::ObjectStorageConstants).
    /// 2) `object_id`: The object identifier. Note that this buffer cannot
    ///    reside in shared memory.
    /// 3) `flags`: The [DataFlag](crate::DataFlag) which determine the settings
    ///    under which the object is opened.
    /// 4) `attributes`: A handle on a
    ///    [PersistentObject](crate::PersistentObject) or an initialized
    ///    [TransientObject](crate::TransientObject) from which to take the
    ///    [PersistentObject](crate::PersistentObject) attributes.
    ///    Can be NONE if the [PersistentObject](crate::PersistentObject)
    ///    contains no attribute. For example,if it is a pure data object.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// let mut init_data: [u8; 0] = [0; 0];
    /// match PersistentObject::create(
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_READ | DataFlag::ACCESS_WRITE,
    ///         None,
    ///         &mut init_data) {
    ///     Ok(object) =>
    ///     {
    ///         // ...
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `ItemNotFound`: If the storage denoted by storage_id does not exist.
    /// 2) `Access_Conflict`: If an access right conflict was detected while
    ///    opening the object.
    /// 3) `OutOfMemory`: If there is not enough memory to complete the
    ///    operation.
    /// 4) `StorageNoSpace`: If insufficient space is available to create the
    ///    persistent object.
    /// 5) `CorruptObject`: If the storage is corrupt.
    /// 6) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object_id.len() >
    ///    [MiscellaneousConstants::TeeObjectIdMaxLen](crate::MiscellaneousConstants::TeeObjectIdMaxLen).
    /// 2) If attributes is not NONE and is not a valid handle on an initialized
    ///    object containing the type and attributes of the object to create.
    /// 3) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn create(
        storage_id: ObjectStorageConstants,
        object_id: &[u8],
        flags: DataFlag,
        attributes: Option<ObjectHandle>,
        initial_data: &[u8],
    ) -> Result<Self> {
        let mut handle: raw::TEE_ObjectHandle = core::ptr::null_mut();
        // Move as much code as possible out of unsafe blocks to maximize Rust’s
        // safety checks.
        let handle_mut = &mut handle;
        let attributes = match attributes {
            Some(a) => a.handle(),
            None => core::ptr::null_mut(),
        };
        match unsafe {
            raw::TEE_CreatePersistentObject(
                storage_id as u32,
                object_id.as_ptr() as _,
                object_id.len(),
                flags.bits(),
                attributes,
                initial_data.as_ptr() as _,
                initial_data.len(),
                handle_mut,
            )
        } {
            raw::TEE_SUCCESS => Ok(Self(ObjectHandle::from_raw(handle)?)),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Marks an object for deletion and closes the object.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_READ) {
    ///     Ok(mut object) =>
    ///     {
    ///         object.close_and_delete()?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid opened object.
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    ///
    /// # Breaking Changes
    ///
    /// Now we no longer need to call `core::mem::forget` after successfully calling
    /// `close_and_delete`, and code like this will now produce a compilation error.
    /// ``` rust,compile_fail
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// # let obj_id = [0_u8];
    /// let mut obj = PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_READ,
    /// )?;
    /// obj.close_and_delete()?;
    /// core::mem::forget(obj); // will get compilation error in this line
    /// //                ^^^ value used here after move
    /// # Ok(())
    /// # }
    pub fn close_and_delete(self) -> Result<()> {
        let result = match unsafe { raw::TEE_CloseAndDeletePersistentObject1(self.0.handle()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        };
        // According to `GPD_TEE_Internal_Core_API_Specification_v1.3.1`:
        // At 5.7.4 TEE_CloseAndDeletePersistentObject1:
        // Deleting an object is atomic; once this function returns, the object
        // is definitely deleted and no more open handles for the object exist.
        //
        // So we must forget the raw_handle to prevent calling TEE_CloseObject
        // on it (no matter the result of TEE_CloseAndDeletePersistentObject1).
        self.0.forget();
        result
    }

    /// Changes the identifier of an object.
    /// The object SHALL have been opened with the
    /// [DataFlag::ACCESS_WRITE_META](crate::DataFlag::ACCESS_WRITE_META) right,
    /// which means access to the object is exclusive.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// let new_obj_id = [2u8;1];
    /// match PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_WRITE_META) {
    ///     Ok(mut object) =>
    ///     {
    ///         object.rename(&new_obj_id)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `AccessConflict`: If an object with the same identifier already
    ///    exists.
    /// 2) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 3) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid handle on a persistent object that has been
    ///    opened with the write-meta access right.
    /// 2) If new_object_id resides in shared memory.
    /// 3) If new_object_id.len() >
    ///    [MiscellaneousConstants::TeeObjectIdMaxLen](crate::MiscellaneousConstants::TeeObjectIdMaxLen).
    /// 4) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn rename(&mut self, new_object_id: &[u8]) -> Result<()> {
        match unsafe {
            raw::TEE_RenamePersistentObject(
                self.0.handle(),
                new_object_id.as_ptr() as _,
                new_object_id.len(),
            )
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Read requested size from the data stream associate with the object into
    /// the buffer.
    ///
    /// # Parameters
    ///
    /// 1) `buffer`: A pre-allocated buffer for saving the object's data stream.
    /// 2) `count`: The returned value contains the number of bytes read.
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_READ) {
    ///     Ok(object) =>
    ///     {
    ///         let mut read_buf = [0u8;16];
    ///         object.read(&mut read_buf)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 2) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid handle on a persistent object opened with
    ///    the read access right.
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn read(&self, buf: &mut [u8]) -> Result<u32> {
        let mut count: usize = 0;
        match unsafe {
            raw::TEE_ReadObjectData(self.handle(), buf.as_mut_ptr() as _, buf.len(), &mut count)
        } {
            raw::TEE_SUCCESS => Ok(count as u32),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Write the passed in buffer data into from the data stream associate with
    /// the object.
    ///
    /// # Parameters
    ///
    /// 1) `buffer`: A pre-allocated buffer for saving the object's data stream.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_WRITE) {
    ///     Ok(mut object) =>
    ///     {
    ///         let write_buf = [1u8;16];
    ///         object.write(& write_buf)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `StorageNoSpace`: If insufficient storage space is available.
    /// 2) `Overflow`: If the value of the data position indicator resulting
    ///    from this operation would be greater than
    ///    [MiscellaneousConstants::TeeDataMaxPosition](crate::MiscellaneousConstants::TeeDataMaxPosition).
    /// 3) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 4) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid handle on a persistent object opened with
    ///    the write access right
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        match unsafe { raw::TEE_WriteObjectData(self.handle(), buf.as_ptr() as _, buf.len()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Change the size of a data stream associate with the object.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open (
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_WRITE) {
    ///     Ok(object) =>
    ///     {
    ///         object.truncate(1u32)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `StorageNoSpace`: If insufficient storage space is available.
    /// 2) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 3) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid handle on a persistent object opened with
    ///    the write access right.
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn truncate(&self, size: u32) -> Result<()> {
        match unsafe { raw::TEE_TruncateObjectData(self.handle(), size as usize) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Set the data position indicator associate with the object.
    ///
    /// # Parameters
    /// 1) `whence`: Defined in [Whence](crate::Whence).
    /// 2) `offset`: The bytes shifted based on `whence`.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{PersistentObject, ObjectStorageConstants, DataFlag, Whence};
    /// # fn main() -> optee_utee::Result<()> {
    /// let obj_id = [1u8;1];
    /// match PersistentObject::open(
    ///         ObjectStorageConstants::Private,
    ///         &obj_id,
    ///         DataFlag::ACCESS_WRITE) {
    ///     Ok(object) =>
    ///     {
    ///         object.seek(0i32, Whence::DataSeekSet)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `Overflow`: If data position indicator is greater than
    ///    [MiscellaneousConstants::TeeDataMaxPosition](crate::MiscellaneousConstants::TeeDataMaxPosition).
    /// 2) `CorruptObject`: If the object is corrupt. The object handle SHALL
    ///    behave based on the `gpd.ta.doesNotCloseHandleOnCorruptObject`
    ///    property.
    /// 3) `StorageNotAvailable`: If the object is stored in a storage area
    ///    which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid handle on a persistent object.
    /// 2) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn seek(&self, offset: i32, whence: Whence) -> Result<()> {
        match unsafe { raw::TEE_SeekObjectData(self.handle(), offset.into(), whence.into()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }
}

impl GenericObject for PersistentObject {
    fn handle(&self) -> raw::TEE_ObjectHandle {
        self.0.handle()
    }
}

#[cfg(test)]
mod tests {
    use optee_utee_sys::{mock_api, mock_utils::{SERIAL_TEST_LOCK, object::MockHandle}};

    use super::*;

    #[test]
    // If a persistent object is successfully created, TEE_CloseObject will be
    // called when it is dropped.
    fn test_create_and_drop() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();
        let fn1 = mock_api::TEE_CreatePersistentObject_context();
        let fn2 = mock_api::TEE_CloseObject_context();

        fn1.expect().return_once_st(move |_, _, _, _, _, _, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });
        fn2.expect().return_once_st(move |obj| {
            debug_assert_eq!(obj, handle);
        });

        let _obj = PersistentObject::create(
            ObjectStorageConstants::Private,
            &[],
            DataFlag::ACCESS_WRITE,
            None,
            &[],
        )
        .expect("it should be ok");
    }

    #[test]
    fn test_create_failed() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        static RETURN_CODE: raw::TEE_Result = raw::TEE_ERROR_BAD_STATE;
        let fn1 = mock_api::TEE_CreatePersistentObject_context();

        fn1.expect().return_const_st(RETURN_CODE);

        let err = PersistentObject::create(
            ObjectStorageConstants::Private,
            &[],
            DataFlag::ACCESS_WRITE,
            None,
            &[],
        )
        .expect_err("it should be err");

        assert_eq!(err.raw_code(), RETURN_CODE);
    }

    #[test]
    // If a persistent object successfully `close_and_delete`, it should not
    // call `TEE_CloseObject` anymore.
    fn test_create_and_successfully_close_delete() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();
        let fn1 = mock_api::TEE_CreatePersistentObject_context();
        let fn2 = mock_api::TEE_CloseAndDeletePersistentObject1_context();
       
        fn1.expect().return_once_st(move |_, _, _, _, _, _, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });
        fn2.expect().return_once_st(move |obj| {
            debug_assert_eq!(obj, handle);
            raw::TEE_SUCCESS
        });

        let obj = PersistentObject::create(
            ObjectStorageConstants::Private,
            &[],
            DataFlag::ACCESS_WRITE,
            None,
            &[],
        )
        .expect("it should be ok");

        obj.close_and_delete().expect("it should be ok");
    }

    #[test]
    // Even a persistent object failed at `close_and_delete`, `TEE_CloseObject`
    // should not be called.
    fn test_create_and_failed_close_delete() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();
        let fn1 = mock_api::TEE_CreatePersistentObject_context();
        let fn2 = mock_api::TEE_CloseAndDeletePersistentObject1_context();

        fn1.expect().return_once_st(move |_, _, _, _, _, _, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });
        fn2.expect().return_once_st(move |obj| {
            debug_assert_eq!(obj, handle);
            raw::TEE_ERROR_BUSY
        });

        let obj = PersistentObject::create(
            ObjectStorageConstants::Private,
            &[],
            DataFlag::ACCESS_WRITE,
            None,
            &[],
        )
        .expect("it should be ok");

        obj.close_and_delete().expect_err("it should be err");
    }
}
