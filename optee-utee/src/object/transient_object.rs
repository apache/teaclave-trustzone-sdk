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

use alloc::vec::Vec;

use optee_utee_sys as raw;

use super::{Attribute, GenericObject, ObjectHandle};
use crate::{Error, Result};

/// Define types of [TransientObject](crate::TransientObject) with
/// predefined maximum sizes.
#[repr(u32)]
pub enum TransientObjectType {
    /// 128, 192, or 256 bits
    Aes = 0xA0000010,
    /// Always 64 bits including the parity bits. This gives an effective key
    /// size of 56 bits
    Des = 0xA0000011,
    /// 128 or 192 bits including the parity bits. This gives effective key
    /// sizes of 112 or 168 bits
    Des3 = 0xA0000013,
    /// Between 64 and 512 bits, multiple of 8 bits
    HmacMd5 = 0xA0000001,
    /// Between 80 and 512 bits, multiple of 8 bits
    HmacSha1 = 0xA0000002,
    /// Between 112 and 512 bits, multiple of 8 bits
    HmacSha224 = 0xA0000003,
    /// Between 192 and 1024 bits, multiple of 8 bits
    HmacSha256 = 0xA0000004,
    /// Between 256 and 1024 bits, multiple of 8 bits
    HmacSha384 = 0xA0000005,
    /// Between 256 and 1024 bits, multiple of 8 bits
    HmacSha512 = 0xA0000006,
    /// The number of bits in the modulus. 256, 512, 768, 1024, 1536 and
    /// 2048-bit keys SHALL be supported.
    /// Support for other key sizes including bigger key sizes is
    /// implementation-dependent. Minimum key size is 256 bits
    RsaPublicKey = 0xA0000030,
    /// Same as [RsaPublicKey](crate::TransientObjectType::RsaPublicKey) key
    /// size.
    RsaKeypair = 0xA1000030,
    /// Depends on Algorithm:
    /// 1) [DsaSha1](crate::AlgorithmId::DsaSha1):
    /// Between 512 and 1024 bits, multiple of 64 bits
    /// 2) [DsaSha224](crate::AlgorithmId::DsaSha224): 2048 bits
    /// 3) [DsaSha256](crate::AlgorithmId::DsaSha256): 2048 or 3072 bits
    DsaPublicKey = 0xA0000031,
    /// Same as [DsaPublicKey](crate::TransientObjectType::DsaPublicKey) key
    /// size.
    DsaKeypair = 0xA1000031,
    /// From 256 to 2048 bits, multiple of 8 bits.
    DhKeypair = 0xA1000032,
    /// Between 160 and 521 bits. Conditional: Available only if at least
    /// one of the ECC the curves defined in Table 6-14 with "generic"
    /// equal to "Y" is supported.
    EcdsaPublicKey = 0xA0000041,
    /// Between 160 and 521 bits. Conditional: Available only if at least
    /// one of the ECC curves defined in Table 6-14 with "generic" equal to
    /// "Y" is supported. SHALL be same value as for ECDSA public key size.
    EcdsaKeypair = 0xA1000041,
    /// Between 160 and 521 bits. Conditional: Available only if at least
    /// one of the ECC curves defined in Table 6-14 with "generic" equal to
    /// "Y" is supported.
    EcdhPublicKey = 0xA0000042,
    /// Between 160 and 521 bits. Conditional: Available only if at least
    /// one of the ECC curves defined in Table 6-14 with "generic" equal to
    /// "Y" is supported. SHALL be same value as for ECDH public key size
    EcdhKeypair = 0xA1000042,
    /// 256 bits. Conditional: Available only if TEE_ECC_CURVE_25519
    /// defined in Table 6-14 is supported.
    Ed25519PublicKey = 0xA0000043,
    /// 256 bits. Conditional: Available only if TEE_ECC_CURVE_25519
    /// defined in Table 6-14 is supported.
    Ed25519Keypair = 0xA1000043,
    /// 256 bits. Conditional: Available only if TEE_ECC_CURVE_25519
    /// defined in Table 6-14 is supported.
    X25519PublicKey = 0xA0000044,
    /// 256 bits. Conditional: Available only if TEE_ECC_CURVE_25519
    /// defined in Table 6-14 is supported.
    X25519Keypair = 0xA1000044,
    /// Multiple of 8 bits, up to 4096 bits. This type is intended for secret
    /// data that has been derived from a key derivation scheme.
    GenericSecret = 0xA0000000,
    /// Object is corrupted.
    CorruptedObject = 0xA00000BE,
    /// 0 – All data is in the associated data stream.
    Data = 0xA00000BF,
}

/// An object containing attributes but no data stream, which is reclaimed
/// when closed or when the TA instance is destroyed.
/// Transient objects are used to hold a cryptographic object (key or key-pair).
///
/// Contrast [PersistentObject](crate::PersistentObject).
#[derive(Debug)]
pub struct TransientObject(ObjectHandle);

impl TransientObject {
    /// Create an object with a null handle which points to nothing.
    //
    // TODO: This function is only used in examples and should be removed when
    // TransientObject is fully refactored in the future. Keep it for now and
    // leave it for future PR.
    pub fn null_object() -> Self {
        Self(ObjectHandle::new_null())
    }

    /// Check if current object is created with null handle.
    ///
    /// # See Also
    ///
    /// - [Self::null_object](Self::null_object).
    pub fn is_null_object(&self) -> bool {
        self.0.is_null()
    }

    /// Allocate an uninitialized object, i.e. a container for attributes.
    ///
    /// As allocated, the object is uninitialized.
    /// It can be initialized by subsequently importing the object material,
    /// generating an object, deriving an object, or loading an object from the
    /// Trusted Storage.
    ///
    /// # Parameters
    ///
    /// 1) `object_type`: Type of uninitialized object container to be created
    ///    as defined in [TransientObjectType](crate::TransientObjectType).
    /// 2) `max_object_size`: Key Size of the object. Valid values depend on the
    ///    object type and are defined in
    ///    [TransientObjectType](crate::TransientObjectType).
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{TransientObject, TransientObjectType};
    /// # fn main() -> optee_utee::Result<()> {
    /// match TransientObject::allocate(TransientObjectType::Aes, 128) {
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
    /// 1) `OutOfMemory`: If not enough resources are available to allocate the
    ///    object handle.
    /// 2) `NotSupported`: If the key size is not supported or the object type
    ///    is not supported.
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error associated with this function
    ///    which is not explicitly associated with a defined return code for
    ///    this function.
    pub fn allocate(object_type: TransientObjectType, max_object_size: usize) -> Result<Self> {
        let mut handle: raw::TEE_ObjectHandle = core::ptr::null_mut();
        // Move as much code as possible out of unsafe blocks to maximize Rust’s
        // safety checks.
        let handle_mut = &mut handle;
        match unsafe {
            raw::TEE_AllocateTransientObject(object_type as u32, max_object_size as u32, handle_mut)
        } {
            raw::TEE_SUCCESS => Ok(Self(ObjectHandle::from_raw(handle)?)),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Reset the object to its initial state after allocation.
    /// If the object is currently initialized, the function clears the object
    /// of all its material.
    /// The object is then uninitialized again.
    pub fn reset(&mut self) {
        unsafe {
            raw::TEE_ResetTransientObject(self.handle());
        }
    }

    /// Populate an uninitialized object container with object attributes passed
    /// by the TA in the `attrs` parameter.
    /// When this function is called, the object SHALL be uninitialized.
    /// If the object is initialized, the caller SHALL first clear it using the
    /// function reset.
    /// Note that if the object type is a key-pair, then this function sets both
    /// the private and public attributes of the keypair.
    ///
    /// # Parameters
    ///
    /// 1) `attrs`: Array of object attributes.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{
    /// #     TransientObject,
    /// #     TransientObjectType,
    /// #     AttributeMemref,
    /// #     AttributeId,
    /// # };
    /// # fn main() -> optee_utee::Result<()> {
    /// match TransientObject::allocate(TransientObjectType::Aes, 128) {
    ///     Ok(mut object) =>
    ///     {
    ///         let attrs = [AttributeMemref::from_ref(AttributeId::SecretValue, &[0u8;1]).into()];
    ///         object.populate(&attrs);
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `BadParameters`: If an incorrect or inconsistent attribute value is
    ///    detected. In this case, the content of the object SHALL remain
    ///    uninitialized.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid opened object that is transient and
    ///    uninitialized.
    /// 2) If some mandatory attribute is missing.
    /// 3) If an attribute which is not defined for the object’s type is
    ///    present in attrs.
    /// 4) If an attribute value is too big to fit within the maximum object
    ///    size specified when the object was created.
    /// 5) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn populate(&mut self, attrs: &[Attribute]) -> Result<()> {
        let p: Vec<raw::TEE_Attribute> = attrs.iter().map(|p| p.raw()).collect();
        match unsafe {
            raw::TEE_PopulateTransientObject(self.0.handle(), p.as_ptr() as _, attrs.len() as u32)
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Populates an uninitialized object handle with the attributes of another
    /// object handle;
    /// that is, it populates the attributes of this handle with the attributes
    /// of src_handle.
    /// It is most useful in the following situations:
    /// 1) To extract the public key attributes from a key-pair object.
    /// 2) To copy the attributes from a
    ///    [PersistentObject](crate::PersistentObject) into a
    ///    [TransientObject](crate::TransientObject).
    ///
    /// # Parameters
    ///
    /// 1) `src_object`: Can be either a
    ///    [TransientObject](crate::TransientObject) or
    ///    [PersistentObject](crate::PersistentObject).
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{TransientObject, TransientObjectType};
    /// # fn main() -> optee_utee::Result<()> {
    /// match TransientObject::allocate(TransientObjectType::Aes, 128) {
    ///     Ok(mut object1) =>
    ///     {
    ///         match TransientObject::allocate(TransientObjectType::Aes, 256) {
    ///             Ok(object2) => {
    ///                 object1.copy_attribute_from(&object2);
    ///                 Ok(())
    ///             }
    ///             Err(e) => Err(e),
    ///         }
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `CorruptObject`: If the persistent object is corrupt. The object
    ///    handle SHALL behave based on the
    ///    `gpd.ta.doesNotCloseHandleOnCorruptObject` property.
    /// 2) `StorageNotAvailable`: If the persistent object is stored in a
    ///    storage area which is currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If src_object is not initialized.
    /// 2) If self is initialized.
    /// 3) If the type and size of src_object and self are not compatible.
    /// 4) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn copy_attribute_from<T: GenericObject>(&mut self, src_object: &T) -> Result<()> {
        match unsafe { raw::TEE_CopyObjectAttributes1(self.handle(), src_object.handle()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Generates a random key or a key-pair and populates a transient key
    /// object with the generated key material.
    ///
    /// # Parameters
    ///
    /// 1) `key_size`: the size of the desired key. It SHALL be less than or
    ///    equal to the maximum key size specified when the transient object
    ///    was created.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{TransientObject, TransientObjectType};
    /// # fn main() -> optee_utee::Result<()> {
    /// match TransientObject::allocate(TransientObjectType::Aes, 128) {
    ///     Ok(object) =>
    ///     {
    ///         object.generate_key(128, &[])?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `BadParameters`: If an incorrect or inconsistent attribute value is detected. In this
    ///    case, the content of the object SHALL remain uninitialized.
    ///
    /// # Panics
    ///
    /// 1) If object is not a valid opened object.
    /// 2) If some mandatory attribute is missing.
    /// 3) If an attribute which is not defined for the object’s type is present
    ///    in attrs.
    /// 4) If an attribute value is too big to fit within the maximum object
    ///    size specified when the object was created.
    /// 5) If the Implementation detects any other error associated with this
    ///    function which is not explicitly associated with a defined return
    ///    code for this function.
    pub fn generate_key(&self, key_size: usize, params: &[Attribute]) -> Result<()> {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        unsafe {
            match raw::TEE_GenerateKey(
                self.handle(),
                key_size as u32,
                p.as_slice().as_ptr() as _,
                p.len() as u32,
            ) {
                raw::TEE_SUCCESS => Ok(()),
                code => Err(Error::from_raw_error(code)),
            }
        }
    }
}

impl GenericObject for TransientObject {
    fn handle(&self) -> raw::TEE_ObjectHandle {
        self.0.handle()
    }
}

#[cfg(test)]
mod tests {
    use optee_utee_sys::{
        mock_api,
        mock_utils::{SERIAL_TEST_LOCK, object::MockHandle},
    };

    use super::*;

    #[test]
    // If a transient object is successfully allocated, TEE_CloseObject will be
    // called when it is dropped.
    //
    // According to `GPD_TEE_Internal_Core_API_Specification_v1.3.1`:
    // At 5.5.5 TEE_CloseObject:
    // The `TEE_CloseObject` function closes an opened object handle. The object
    // can be persistent or transient.
    // For transient objects, `TEE_CloseObject` is equivalent to
    // `TEE_FreeTransientObject`.
    fn test_allocate_and_drop() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let mut raw_handle = MockHandle::new();
        let handle = raw_handle.as_handle();
        let fn1 = mock_api::TEE_AllocateTransientObject_context();
        let fn2 = mock_api::TEE_CloseObject_context();

        fn1.expect().return_once_st(move |_, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });
        fn2.expect().return_once_st(move |obj| {
            debug_assert_eq!(obj, handle.clone());
        });

        let _obj =
            TransientObject::allocate(TransientObjectType::Aes, 128).expect("it should be ok");
    }

    #[test]
    fn test_allocate_fail() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        static RETURN_CODE: raw::TEE_Result = raw::TEE_ERROR_BAD_STATE;
        let fn1 = mock_api::TEE_AllocateTransientObject_context();

        fn1.expect().return_const(RETURN_CODE);

        let err =
            TransientObject::allocate(TransientObjectType::Aes, 128).expect_err("it should be err");

        assert_eq!(err.raw_code(), RETURN_CODE);
    }
}
