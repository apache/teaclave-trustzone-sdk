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

use alloc::{boxed::Box, vec::Vec};
use core::{mem, ptr};

use optee_utee_sys as raw;

use crate::{Attribute, Error, GenericObject, Result, TransientObject};

/// Specify one of the available cryptographic operations.
#[repr(u32)]
pub enum OperationMode {
    /// Encryption mode
    Encrypt = 0,
    /// Decryption mode
    Decrypt = 1,
    /// Signature generation mode
    Sign = 2,
    /// Signature verfication mode
    Verify = 3,
    /// MAC mode
    Mac = 4,
    /// Digest mode
    Digest = 5,
    /// Key derivation mode
    Derive = 6,
    /// Reserve for testing and validation
    IllegalValue = 0x7fffffff,
}

/// Represent the information about a crypto information.
pub struct OperationInfo {
    raw: raw::TEE_OperationInfo,
}

impl OperationInfo {
    /// Return the `OperationInfo` struct based on the raw struct `TEE_OperationInfo`.
    ///
    /// The raw structure contains following fields:
    ///
    /// 1) `algorithm`: One of the algorithm of [AlgorithmId](AlgorithmId).
    /// 2) `mode`: One of the mode of [OperationMode](OperationMode).
    /// 3) `maxKeySize`: The maximum key sizes of different algorithms as defined in
    /// [TransientObjectType](../object/enum.TransientObjectType.html).
    /// 4) `operationClass`: One of the constants from [OperationConstant](OperationConstant).
    /// 5) `keySize`:
    /// 5.1) For an operation that makes no use of keys, 0.
    /// 5.2) For an operation that uses a single key, the actual size of this key.
    /// 5.3) For an operation that uses multiple keys, 0. (The actual value of `keySize` can be obtained from
    /// [OperationInfoMultiple](OperationInfoMultiple)).
    /// 6) `requiredKeyUsage`:
    /// 6.1) For an operation that makes no use of keys, 0.
    /// 6.2) For an operation that uses a single key, a bit vector that describes the necessary bits in the object
    /// usage for `set_key` functions to succeed without panicking.
    /// 6.3) For an operation that uses multiple keys, 0. (The actual value of `requiredKeyUsage` can be obtained from
    /// [OperationInfoMultiple](OperationInfoMultiple).
    /// 7) `digestLength`: For a [Mac](Mac), [AE](AE), or [Digest](Digest), describes the number of bytes in the digest or tag.
    /// 8) `handleState`: A bit vector describing the current state of the operation. Contains one or more of the
    /// [HandleFlag](../object/struct.HandleFlag.html).
    pub fn from_raw(raw: raw::TEE_OperationInfo) -> Self {
        Self { raw }
    }

    /// Return the `keySize` field of the raw structure `TEE_OperationInfo`.
    pub fn key_size(&self) -> u32 {
        self.raw.keySize
    }

    /// Return the `maxDataSize` field of the raw structure `TEE_OperationInfo`.
    pub fn max_key_size(&self) -> u32 {
        self.raw.maxKeySize
    }
}

/// Every operation of [AE](AE), [Asymmetric](Asymmetric), [Cipher](Cipher),
/// [DeriveKey](DeriveKey), [Digest](Digest), [Mac](Mac) can be either one of the two states.
#[repr(u32)]
pub enum OperationStates {
    /// Nothing is going on.
    Initial = 0x00000000,
    /// An operation is in progress.
    Active = 0x00000001,
}

/// Define the supported crypto operation.
pub enum OperationConstant {
    /// [Cipher](Cipher)
    Cipher = 1,
    /// [Mac](Mac)
    Mac = 3,
    /// [AE](AE)
    Ae = 4,
    /// [Digest](Digest)
    Digest = 5,
    /// [Asymmetric](Asymmetric)
    AsymmetricCipher = 6,
    /// [Asymmetric](Asymmetric)
    AsymmetricSignature = 7,
    /// [DeriveKey](DeriveKey)
    KeyDerivation = 8,
}

/// Represent the information about a crypto information which uses multiple keys.
pub struct OperationInfoMultiple {
    raw: *mut raw::TEE_OperationInfoMultiple,
    size: usize,
}

impl OperationInfoMultiple {
    /// Return the `OperationInfoMultiple` struct based on the raw struct `TEE_OperationInfo`.
    ///
    /// The raw structure contains following fields:
    ///
    /// 1) `algorithm`: One of the algorithm of [AlgorithmId](AlgorithmId).
    /// 2) `mode`: One of the mode of [OperationMode](OperationMode).
    /// 3) `maxKeySize`: The maximum key sizes of different algorithms as defined in
    /// [TransientObjectType](../object/enum.TransientObjectType.html).
    /// 4) `operationClass`: One of the constants from [OperationConstant](OperationConstant).
    /// 5) `digestLength`: For a [Mac](Mac), [AE](AE), or [Digest](Digest), describes the number of bytes in the digest or tag.
    /// 6) `handleState`: A bit vector describing the current state of the operation. Contains one or more of the [HandleFlag](../object/struct.HandleFlag.html).
    /// 7) `operationState`: Every operation has two states which are defined as
    ///    [OperationStates](OperationStates).
    /// 8) `numberOfKeys`: This is set to the number of keys required by this operation. May be 0 for an operation which requires no keys.
    /// 9) `keyInformation`: This array contains numberOfKeys entries, each of which defines the details for one key used by the operation,
    /// in the order they are defined.
    /// If the buffer is larger than required to support `numberOfKeys` entries, the additional space is not initialized or modified.
    /// For each element:
    /// 9.1) `keySize`: If a key is programmed in the operation, the actual size of this key, otherwise 0.
    /// 9.2) `requiredKeyUsage`: A bit vector that describes the necessary bits in the object usage for `set_key` or `set_key_2` to succeed without panicking.
    pub fn from_raw(raw: *mut raw::TEE_OperationInfoMultiple, size: usize) -> Self {
        Self { raw, size }
    }

    /// Return the raw struct `TEE_OperationInfoMultiple`.
    pub fn raw(&self) -> *mut raw::TEE_OperationInfoMultiple {
        self.raw
    }

    /// Return the `size` field of the raw structure `TEE_OperationInfoMultiple`.
    pub fn size(&self) -> usize {
        self.size
    }
}

/// An opaque reference that identifies a particular cryptographic operation.
pub struct OperationHandle {
    raw: *mut raw::TEE_OperationHandle,
}

impl OperationHandle {
    fn from_raw(raw: *mut raw::TEE_OperationHandle) -> OperationHandle {
        Self { raw }
    }

    fn handle(&self) -> raw::TEE_OperationHandle {
        unsafe { *(self.raw) }
    }

    fn null() -> Self {
        OperationHandle::from_raw(ptr::null_mut())
    }

    fn allocate(algo: AlgorithmId, mode: OperationMode, max_key_size: usize) -> Result<Self> {
        let raw_handle: *mut raw::TEE_OperationHandle = Box::into_raw(Box::new(ptr::null_mut()));
        match unsafe {
            raw::TEE_AllocateOperation(
                raw_handle as *mut _,
                algo as u32,
                mode as u32,
                max_key_size as u32,
            )
        } {
            raw::TEE_SUCCESS => Ok(Self::from_raw(raw_handle)),
            code => Err(Error::from_raw_error(code)),
        }
    }

    fn info(&self) -> OperationInfo {
        let mut raw_info: raw::TEE_OperationInfo = unsafe { mem::zeroed() };
        unsafe { raw::TEE_GetOperationInfo(self.handle(), &mut raw_info) };
        OperationInfo::from_raw(raw_info)
    }

    fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        let mut tmp_size: usize = 0;
        match unsafe {
            raw::TEE_GetOperationInfoMultiple(self.handle(), info_buf.as_ptr() as _, &mut tmp_size)
        } {
            raw::TEE_SUCCESS => Ok(OperationInfoMultiple::from_raw(
                info_buf.as_ptr() as _,
                tmp_size,
            )),
            code => Err(Error::from_raw_error(code)),
        }
    }

    fn reset(&mut self) {
        unsafe {
            raw::TEE_ResetOperation(self.handle());
        }
    }

    fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        match unsafe { raw::TEE_SetOperationKey(self.handle(), object.handle()) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    fn copy<T: OpHandle>(&mut self, src: &T) {
        unsafe {
            raw::TEE_CopyOperation(self.handle(), src.handle());
        }
    }
}

/// determine whether a combination of algId and element is supported
pub fn is_algorithm_supported(alg_id: u32, element: u32) -> Result<()> {
    match unsafe { raw::TEE_IsAlgorithmSupported(alg_id, element) } {
        raw::TEE_SUCCESS => Ok(()),
        code => Err(Error::from_raw_error(code)),
    }
}

// free before check it's not null
/// Deallocate all resources associated with an operation handle. After this function is called,
/// the operation handle is no longer valid. All cryptographic material in the operation is destroyed.
impl Drop for OperationHandle {
    fn drop(&mut self) {
        unsafe {
            if !self.raw.is_null() {
                raw::TEE_FreeOperation(self.handle());
            }
            drop(Box::from_raw(self.raw));
        }
    }
}

/// A trait for a crypto operation to return its handle.
pub trait OpHandle {
    /// Return the handle of an operation.
    fn handle(&self) -> raw::TEE_OperationHandle;
}

/// An operation for digest the message.
pub struct Digest(OperationHandle);

impl Digest {
    /// Accumulate message data for hashing. The message does not have to be block aligned.
    /// Subsequent calls to this function are possible. The operation may be in either
    /// initial or active state and becomes active.
    ///
    /// # Parameters
    ///
    /// 1) `chunk`: Chunk of data to be hashed
    ///
    /// # Panics
    ///
    /// 1) If the operation is not allocated with valid algorithms.
    /// 2) if input data exceeds maximum length for algorithm.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn update(&self, chunk: &[u8]) {
        unsafe {
            raw::TEE_DigestUpdate(self.handle(), chunk.as_ptr() as _, chunk.len());
        }
    }

    /// Finalize the message digest operation and produces the message hash. Afterwards the
    /// Message Digest operation is reset to initial state and can be reused.
    ///
    /// # Parameters
    ///
    /// 1) `chunk`: Last chunk of data to be hashed.
    /// 2) `hash`: Output buffer filled with the message hash. This buffer should be large enough to
    ///    hold the hash message. The real used size is returned by this function.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Digest, AlgorithmId};
    /// # fn main() -> optee_utee::Result<()> {
    /// let chunk1 = [0u8;8];
    /// let chunk2 = [1u8;8];
    /// let mut hash = [0u8;32];
    /// match Digest::allocate(AlgorithmId::Sha256) {
    ///     Ok(operation) =>
    ///     {
    ///         operation.update(&chunk1);
    ///         match operation.do_final(&chunk2, &mut hash) {
    ///             Ok(hash_len) => {
    ///                 // ...
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
    /// 1) `ShortBuffer`: If the `hash` is too small. Operation is not finalized for this error.
    ///
    /// # Panics
    /// 1) If the operation is not allocated with valid algorithms.
    /// 2) if input data exceeds maximum length for algorithm.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    //hash size is dynamic changed so we returned it's updated size
    pub fn do_final(&self, chunk: &[u8], hash: &mut [u8]) -> Result<usize> {
        let mut hash_size: usize = hash.len();
        match unsafe {
            raw::TEE_DigestDoFinal(
                self.handle(),
                chunk.as_ptr() as _,
                chunk.len(),
                hash.as_mut_ptr() as _,
                &mut hash_size,
            )
        } {
            raw::TEE_SUCCESS => Ok(hash_size),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create a Digest operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Allocate a new cryptographic operation and sets the mode and algorithm type.
    ///
    /// # Parameters
    ///
    /// 1) `algo`: One of the algorithms that support Digest as listed in
    ///    [AlgorithmId](AlgorithmId).
    /// 2) `max_key_size`: The maximum key sizes of different algorithms as defined in
    ///    [TransientObjectType](../object/enum.TransientObjectType.html).
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Digest, AlgorithmId};
    /// # fn main() -> optee_utee::Result<()> {
    /// match Digest::allocate(AlgorithmId::Sha256) {
    ///     Ok(operation) =>
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
    /// 1) `OutOfMemory`: If not enough resources are available to allocate the object handle.
    /// 2) `NotSupported`: If the key size is not supported or the object type is not supported.
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error associated with this function which is not
    /// explicitly associated with a defined return code for this function.
    pub fn allocate(algo: AlgorithmId) -> Result<Self> {
        match OperationHandle::allocate(algo, OperationMode::Digest, 0) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Return the characteristics of a Digest operation.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Digest, AlgorithmId};
    /// # fn main() -> optee_utee::Result<()> {
    /// match Digest::allocate(AlgorithmId::Md5) {
    ///     Ok(operation) =>
    ///     {
    ///         let info = operation.info();
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    /// 1) If the operation is not a valid opened operation.
    /// 2) if the Implementation detecs any other error.
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Return the characteristics of a Digest operation with multiple keys.
    ///
    /// # Parameters
    ///
    /// 1) `info_buf`: The buffer is supposed to save multiple keys, and its size should be large enough before passed in.
    /// The number of keys about this operation can be calculated as: OperationInfoMultiple::size -
    /// size_of([OperationInfoMultiple](OperationInfoMultiple)) / size_of ( raw::TEE_OperationInfoKey)+1.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Digest, AlgorithmId};
    /// # fn main() -> optee_utee::Result<()> {
    /// match Digest::allocate(AlgorithmId::Md5) {
    ///     Ok(operation) =>
    ///     {
    ///         let mut buffer = [0u8, 12];
    ///         match operation.info_multiple(&mut buffer) {
    ///             Ok(info_multiple) => {
    ///                 // ...
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
    /// # Errors:
    ///
    /// 1) `ShortBuffer`: If the `info_buf` is not large enough to hold an
    ///    [OperationInfoMultiple](OperationInfoMultiple) and the corresponding keys.
    ///
    /// # Panics:
    ///
    /// 1) If operation is not a valid opened object.
    /// 2) If the Implementation detects any other error.
    // Here the multiple info total size is not sure
    // Passed in array is supposed to provide enough size for this struct
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Reset the operation state to the state after initial [allocate](Digest::allocate) with the
    /// add addition of any keys which were configured subsequent to this so that current operation
    /// can be reused with the same keys.
    ///
    /// # Panics
    ///
    /// 1) If operation is not a valid opened object.
    /// 2) If the key has not been set yet.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn reset(&mut self) {
        self.0.reset()
    }

    /// Copy an operation state to another operation. This also copies the key material associated
    /// with the source operation.
    ///
    /// # Parameters
    ///
    /// 1) `src`: the source operation.
    /// 1.1) If `src` has no key programmed, then the key of this operation is cleared. If there is a key
    /// programmed in srcOperation, then the maximum key size of current SHALL be greater than or
    /// equal to the actual key size of src.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Digest, AlgorithmId};
    /// # fn main() -> optee_utee::Result<()> {
    /// match Digest::allocate(AlgorithmId::Sha256) {
    ///     Ok(mut operation) =>
    ///     {
    ///         match Digest::allocate(AlgorithmId::Sha256) {
    ///             Ok(operation2) =>
    ///             {
    ///                 // ...
    ///                 operation.copy(&operation2);
    ///                 Ok(())
    ///             },
    ///             Err(e) => Err(e),
    ///         }
    ///     },
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// 1) If the operation or source operation is not a valid opened operation.
    /// 2) If the alogirhtm or mode differe in two perations.
    /// 3) If `src` has akey and its size is greater than the maximum key size of the operation.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for Digest {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for conducting symmetric cipher encryption / decryption.
/// This operation defines the way to perform symmetric cipher operations, such as AES.
/// They cover both block ciphers and stream ciphers.
pub struct Cipher(OperationHandle);

impl Cipher {
    /// Start the symmetric cipher operation. The function should be called after the
    /// [set_key](Cipher::set_key) or [set_key_2](Cipher::set_key_2).
    ///
    /// After called, if the operation is in active state, it is reset and then initialized.
    /// If the operation is in initial state, it is moved to active state.
    ///
    /// # Parameters
    ///
    /// 1) `iv`: buffer contains the operation Initialization Vector, which is used for:
    /// 1.1) [AesCbcNopad](AlgorithmId::AesCbcNopad): IV;
    /// 1.2) [AesCtr](AlgorithmId::AesCtr): Initial Counter Value;
    /// 1.3) [AesCts](AlgorithmId::AesCts): IV;
    /// 1.4) [AesXts](AlgorithmId::AesXts): Tweak Value;
    /// 1.5) [AesCcm](AlgorithmId::AesCcm): Nonce Value;
    /// 1.6) [AesGcm](AlgorithmId::AesGcm): Nonce Value;
    /// 1.7) [AesCbcNopad](AlgorithmId::AesCbcNopad): IV.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Cipher`.
    /// 2) If no key is programmed in the operation.
    /// 3) If the IV does not have the length required by the algorithm.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn init(&self, iv: &[u8]) {
        unsafe { raw::TEE_CipherInit(self.handle(), iv.as_ptr() as _, iv.len()) };
    }

    /// Encrypt or decrypt the source data.
    ///
    /// Input data does not have to be a multiple of block size. Subsequent calls to this function are possible.
    /// Unless one or more calls of this function have supplied sufficient input data, no output is generated.
    /// The function should be called after the [init](Cipher::init).
    ///
    /// # Parameters
    ///
    /// 1) `src`: Input data buffer to be encrypted or decrypted.
    /// 2) `dest`: Output buffer.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Cipher, AlgorithmId, TransientObject, TransientObjectType};
    /// # use optee_utee::{AttributeMemref, AttributeId, OperationMode};
    /// # fn main() -> optee_utee::Result<()> {
    /// let iv = [0u8, 16];
    /// let key = [0u8, 16];
    /// let src = [1u8; 4096];
    /// let mut dest = [0u8; 4096];
    /// match Cipher::allocate(AlgorithmId::AesCtr, OperationMode::Encrypt, 128) {
    ///     Ok(operation) =>
    ///     {
    ///         match TransientObject::allocate(TransientObjectType::Aes, 128) {
    ///             Ok(mut object) =>
    ///             {
    ///                 let attr = AttributeMemref::from_ref(AttributeId::SecretValue, &key);
    ///                 object.populate(&[attr.into()])?;
    ///                 operation.set_key(&object)?;
    ///                 operation.init(&iv);
    ///                 operation.update(&src, &mut dest)?;
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
    /// 1) `ShortBuffer`: If the output buffer is not large enough to contain the output.
    /// In this case, the input is not fed into the algorithm.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Cipher`.
    /// 2) If the function is called before [init](Cipher::init) or after
    ///    [do_final](Cipher::do_final).
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn update(&self, src: &[u8], dest: &mut [u8]) -> Result<usize> {
        let mut dest_size: usize = dest.len();
        match unsafe {
            raw::TEE_CipherUpdate(
                self.handle(),
                src.as_ptr() as _,
                src.len(),
                dest.as_mut_ptr() as _,
                &mut dest_size,
            )
        } {
            raw::TEE_SUCCESS => {
                Ok(dest_size)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Finalize the cipher operation, processing data that has not been processed by previous calls
    /// to [update](Cipher::update) as well as data supplied in `src`. The operation handle can be reused or re-initialized.
    ///
    /// # Parameters
    ///
    /// 1) `src`: Input data buffer to be encrypted or decrypted.
    /// 2) `dest`: Output buffer.
    ///
    /// # Errors
    ///
    /// 1) `ShortBuffer`: If the output buffer is not large enough to contain the output.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Cipher`.
    /// 2) If the function is called before [init](Cipher::init).
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn do_final(&self, src: &[u8], dest: &mut [u8]) -> Result<usize> {
        let mut dest_size: usize = dest.len();
        match unsafe {
            raw::TEE_CipherDoFinal(
                self.handle(),
                src.as_ptr() as _,
                src.len(),
                dest.as_mut_ptr() as _,
                &mut dest_size,
            )
        } {
            raw::TEE_SUCCESS => Ok(dest_size),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create a Cipher operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Function usage is similar to [Digest::allocate](Digest::allocate).
    pub fn allocate(algo: AlgorithmId, mode: OperationMode, max_key_size: usize) -> Result<Self> {
        match OperationHandle::allocate(algo, mode, max_key_size) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Function usage is similar to [Digest::info](Digest::info).
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Function usage is similar to [Digest::info_multiple](Digest::info_multiple).
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Program the key of Digest operation. That ids, it associates the operation with a key.
    ///
    /// # Parameters
    ///
    /// 1) `object`: The object can either be a [Transient](../object/struct.TransientObject.html)
    ///    or [Persistent](../object/struct.PersistentObject.html). The key material is copied from
    ///    the key object handle into the operation. After the key has been set, there is no longer
    ///    any link between the operation and the key object. The object handle can be closed or reset
    ///    and this will not affect the operation. This copied material exists until the operation is
    ///    freed or another key is set into the operation.
    ///
    /// # Errors
    ///
    /// 1) `CorruptObject`: If the object is corrupt. The object handle is closed.
    /// 2) `StorageNotAvailable`: If the object is stored in a storage area which is
    ///    currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If operation is not a valid opened object.
    /// 2) If object is not null and is not a valid key object.
    /// 3) If object is not initialized.
    /// 4) If the operation expect two keys as [AesXts](AlgorithmId::AesXts).
    /// 5) If the type, size, or usage of object is not compatible with the algorithm, mode, or size of the operation.
    /// 6) If operation is not in initial state.
    /// 7) Hardware or cryptographic algorithm failure.
    /// 8) If the Implementation detects any other error.
    pub fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        self.0.set_key(object)
    }

    /// Initialize an expisting operation with two keys for [AesXts](AlgorithmId::AesXts).
    ///
    /// # Parameters:
    ///
    /// object1 and object2 SHALL both be non-NULL or both NULL. object1 and object2 SHALL NOT refer to keys with
    /// bitwise identical [SecretValue](../object/enum.AttributeId.html#variant.SecretValue) attributes.
    ///
    /// # Errors
    ///
    /// 1) `CorruptObject`: If the object1 is corrupt. The object handle is closed.
    /// 2) `CorruptObject2`: If the object2 is corrupt. The object handle is closed.
    /// 3) `StorageNotAvailable`: If the object1 is stored in a storage area which is
    ///    currently inaccessible.
    /// 4) `StorageNotAvailable2`: If the object2 is stored in a storage area which is
    ///    currently inaccessible.
    ///
    /// # Panics
    ///
    /// 1) If operation is not a valid opened object.
    /// 2) If object1 and object2 are not both null and object1 or object2 or both are not a valid key object.
    /// 3) If object1 or object2 is not initialized.
    /// 4) If the operation algorithm is not [AesXts](AlgorithmId::AesXts).
    /// 5) If the type, size, or usage of any object is not compatible with the algorithm, mode, or size of the operation.
    /// 6) If operation is not in initial state.
    /// 7) Hardware or cryptographic algorithm failure.
    /// 8) If the Implementation detects any other error.
    pub fn set_key_2<T: GenericObject, D: GenericObject>(&self, object1: &T, object2: &D) -> Result<()> {
        match unsafe {
            raw::TEE_SetOperationKey2(self.handle(), object1.handle(), object2.handle())
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Function usage is similar to [Digest::copy](Digest::copy).
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for Cipher {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for performing MAC (Message Authentication Code) operations, such as `HMAC`
/// or `AES-CMAC` operations. This operation is not used for Authenticated Encryption algorithms,
/// which SHALL use the functions defined in [AE](AE).
pub struct Mac(OperationHandle);

impl Mac {
    /// Initialize a MAC opeartion. The The function should be called after the
    /// [set_key](Mac::set_key).
    ///
    /// # Parameters
    ///
    /// 1) `iv`: Input buffer containing the operation Initialization Vector, if applicable
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Mac`.
    /// 2) If no key is programmed in the operation.
    /// 3) If the Initialization Vector does not have the length required by the algorithm.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn init(&self, iv: &[u8]) {
        unsafe { raw::TEE_MACInit(self.handle(), iv.as_ptr() as _, iv.len()) };
    }

    /// Accumulate data for a MAC calculation.
    ///
    /// Input data does not have to be a multiple of block size. Subsequent calls to this function are possible.
    /// Unless one or more calls of this function have supplied sufficient input data, no output is generated.
    /// The function should be called after the [init](Mac::init).
    ///
    /// # Parameters
    ///
    /// 1) `chunk`: Chunk of the message to be MACed.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Mac`.
    /// 2) If the function is called before [init](Mac::init) or after
    ///    [compute_final](Mac::compute_final) or after [compare_final](Mac::compare_final).
    /// 3) If `chunk` excceds maximum length for algorithm.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn update(&self, chunk: &[u8]) {
        unsafe { raw::TEE_MACUpdate(self.handle(), chunk.as_ptr() as _, chunk.len()) };
    }
    /// Finalize the MAC operation with a last chunk of message, and computes the MAC.
    /// Afterwards the operation handle can be reused or re-initialized with a new key.
    /// The operation SHALL be in active state and moves to initial state afterwards.
    ///
    /// # Parameters:
    ///
    /// `message`: Input buffer containing a last message chunk to MAC
    /// `mac`: Output buffer filled with the computed MAC, the size should be allocated enough for
    /// containing the whole computed MAC
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{
    ///     TransientObject,
    ///     TransientObjectType,
    ///     Attribute,
    ///     AttributeMemref,
    ///     AttributeId,
    ///     Mac,
    ///     AlgorithmId,
    /// };
    /// # fn main() -> optee_utee::Result<()> {
    /// let mut key: [u8; 20] = [
    /// 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35,
    /// 0x36, 0x37, 0x38, 0x39, 0x30,];
    /// let mut out: [u8; 20] = [0u8; 20];
    /// match Mac::allocate(AlgorithmId::HmacSha1, key.len() * 8) {
    ///     Err(e) => return Err(e),
    ///     Ok(mac) => {
    ///         match TransientObject::allocate(TransientObjectType::HmacSha1, key.len() * 8) {
    ///             Err(e) => return Err(e),
    ///             Ok(mut key_object) => {
    ///                 let attr = AttributeMemref::from_ref(AttributeId::SecretValue, &key);
    ///                 key_object.populate(&[attr.into()])?;
    ///                 mac.set_key(&key_object)?;
    ///             }
    ///         }
    ///         mac.init(&[0u8; 0]);
    ///         mac.update(&[0u8; 8]);
    ///         mac.compute_final(&[0u8; 0], &mut out)?;
    ///         Ok(())
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `ShortBuffer`: If the output buffer is not large enough to contain the output.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Mac`.
    /// 2) If the function is called before before [init](Mac::init) or after
    ///    [compute_final](Mac::compute_final) or after [compare_final](Mac::compare_final).
    /// 3) If input data exceeds maximum length for algorithm.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn compute_final(&self, message: &[u8], mac: &mut [u8]) -> Result<usize> {
        let mut mac_size: usize = mac.len();
        match unsafe {
            raw::TEE_MACComputeFinal(
                self.handle(),
                message.as_ptr() as _,
                message.len(),
                mac.as_mut_ptr() as _,
                &mut mac_size,
            )
        } {
            raw::TEE_SUCCESS => Ok(mac_size),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Finalize the MAC operation and compares the MAC with the buffer passed to the function.
    /// Afterwards the operation handle can be reused or re-initialized with a new key.
    /// The operation SHALL be in active state and moves to initial state afterwards.
    ///
    /// # Parameters:
    ///
    /// `message`: Input buffer containing a last message chunk to MAC
    /// `mac`: Input buffer containing the MAC to check
    ///
    /// # Errors
    ///
    /// 1) `MacInvald`: If the computed MAC does not correspond to the value passed in `mac`.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `Mac`.
    /// 2) If operation is not in active state.
    /// 3) If input data exceeds maximum length for algorithm.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn compare_final(&self, message: &[u8], mac: &[u8]) -> Result<()> {
        match unsafe {
            raw::TEE_MACCompareFinal(
                self.handle(),
                message.as_ptr() as _,
                message.len(),
                mac.as_ptr() as _,
                mac.len(),
            )
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create a Mac operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Function usage is similar to [Digest::allocate](Digest::allocate).
    pub fn allocate(algo: AlgorithmId, max_key_size: usize) -> Result<Self> {
        match OperationHandle::allocate(algo, OperationMode::Mac, max_key_size) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Function usage is similar to [Digest::info](Digest::info).
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Function usage is similar to [Digest::info_multiple](Digest::info_multiple).
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Function usage is similar to [Digest::reset](Digest::reset).
    pub fn reset(&mut self) {
        self.0.reset()
    }

    /// Function usage is similar to [Cipher::set_key](Cipher::set_key).
    pub fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        self.0.set_key(object)
    }

    /// Function usage is similar to [Digest::copy](Digest::copy).
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for Mac {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for conducting authenticated encryption / decryption.
pub struct AE(OperationHandle);

impl AE {
    /// Initialize an AE opeartion.
    /// The operation must be in the initial state and remains in the initial state afterwards.
    ///
    /// # Parameters
    ///
    /// 1) `nonce`: The peration nonce or IV
    /// 2) `tag_len`: Size in bits of the tag:
    /// 2.1) for `AES-GCM`, can be 128, 120, 112, 104, or 96;
    /// 2.2) for `AES-CCM`, can be 128, 112, 96, 80, 64, 48, or 32.
    /// 3) `aad_len`: length in bytes of the AAD (Used only for AES-CCM. Ignored for AES-GCM).
    /// 4) `pay_load_len`: Length in bytes of the payload (Used only for AES-CCM. Ignored for AES-GCM).
    ///
    /// # Errors
    ///
    /// 1) `NotSupported`: If the `tag_len` is not supported by the algorithm.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `AE`.
    /// 2) If no key is programmed in the operation.
    /// 3) If the nonce length is not compatible with the length required by the algorithm.
    /// 4) If operation is not in initial state.
    /// 5) Hardware or cryptographic algorithm failure.
    /// 6) If the Implementation detects any other error.
    pub fn init(
        &self,
        nonce: &[u8],
        tag_len: usize,
        aad_len: usize,
        pay_load_len: usize,
    ) -> Result<()> {
        match unsafe {
            raw::TEE_AEInit(
                self.handle(),
                nonce.as_ptr() as _,
                nonce.len(),
                tag_len as u32,
                aad_len,
                pay_load_len,
            )
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Feed a new chunk of Additional Authentication Data (AAD) to the AE operation.
    /// Subsequent calls to this function are possible.
    /// The operation SHALL be in initial state and remains in initial state afterwards.
    ///
    /// # Parameters
    ///
    /// 1) `aad_data`: Input buffer containing the chunk of AAD.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `AE`.
    /// 2) If the function is called before [init](AE::init) or has been finalized.
    /// 3) For `AES-CCM`, if the `aad_data.len()` exceeds the requirement.
    /// 4) If operation is not in initial state.
    /// 5) Hardware or cryptographic algorithm failure.
    /// 6) If the Implementation detects any other error.
    pub fn update_aad(&self, aad_data: &[u8]) {
        unsafe { raw::TEE_AEUpdateAAD(self.handle(), aad_data.as_ptr() as _, aad_data.len()) };
    }

    /// Accumulate data for an Authentication Encryption operation.
    /// Input data does not have to be a multiple of block size. Subsequent calls to this function are possible.
    /// Unless one or more calls of this function have supplied sufficient input data, no output is generated.
    /// The buffers `src` and `dest` SHALL be either completely disjoint or equal in their starting positions.
    /// The operation may be in either initial or active state and enters active state afterwards if `src.len()` != 0.
    ///
    /// # Parameters
    ///
    /// 1) `src`: Input data buffer to be encrypted or decrypted.
    /// 2) `dest`: Output buffer.
    ///
    /// # Errors
    ///
    /// `ShortBuffer`: If the output buffer is not large enough to contain the output.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `AE`.
    /// 2) If the function is called before [init](AE::init) or has been finalized.
    /// 3) For `AES-CCM`, if the AAD length exceeds the requirement.
    /// 4) For `AES-CCM`, if the payload length is exceeds the requirement.
    /// 5) Hardware or cryptographic algorithm failure.
    /// 6) If the Implementation detects any other error.
    pub fn update(&self, src: &[u8], dest: &mut [u8]) -> Result<usize> {
        let mut dest_size: usize = dest.len();
        match unsafe {
            raw::TEE_AEUpdate(
                self.handle(),
                src.as_ptr() as _,
                src.len(),
                dest.as_mut_ptr() as _,
                &mut dest_size,
            )
        } {
            raw::TEE_SUCCESS => {
                Ok(dest_size)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }
    /// Process data that has not been processed by previous calls to [update](AE::update) as well as data supplied in `src`.
    /// It completes the AE operation and computes the tag.
    /// The buffers `src` and `dest` SHALL be either completely disjoint or equal in their starting positions.
    /// The operation may be in either initial or active state and enters initial state afterwards.
    ///
    /// # Parameters
    ///
    /// 1) `src`: Reference to final chunk of input data to be encrypted.
    /// 2) `dest`: Output buffer. Can be omitted if the output is to be discarded, e.g. because it is known to be empty.
    /// 3) `tag`: Output buffer filled with the computed tag.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{AE, AlgorithmId, OperationMode, AttributeMemref, AttributeId};
    /// # use optee_utee::{TransientObject, TransientObjectType};
    /// # fn main() -> optee_utee::Result<()> {
    /// let key = [0xa5u8; 16];
    /// let nonce = [0x00u8; 16];
    /// let aad = [0xffu8; 16];
    /// let clear1 = [0x5au8; 19];
    /// let clear2 = [0xa5u8; 13];
    /// let mut ciph1 = [0x00u8; 16];
    /// let mut ciph2 = [0x00u8; 16];
    /// let mut tag = [0x00u8; 16];
    /// match AE::allocate(AlgorithmId::AesCcm, OperationMode::Encrypt, 128) {
    ///     Ok(operation) => {
    ///         match TransientObject::allocate(TransientObjectType::Aes, 128) {
    ///             Ok(mut key_object) => {
    ///                 let attr = AttributeMemref::from_ref(AttributeId::SecretValue, &key);
    ///                 key_object.populate(&[attr.into()])?;
    ///                 operation.set_key(&key_object)?;
    ///                 operation.init(&nonce, 128, 16, 32)?;
    ///                 operation.update_aad(&aad);
    ///                 operation.update(&clear1, &mut ciph1)?;
    ///                 match operation.encrypt_final(&clear2, &mut ciph2, &mut tag) {
    ///                     Ok((_ciph_len, _tag_len)) => {
    ///                         // ...
    ///                         Ok(())
    ///                     },
    ///                     Err(e) => Err(e),
    ///                 }
    ///             },
    ///             Err(e) => Err(e),
    ///         }
    ///     },
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// `ShortBuffer`: If the output tag buffer is not large enough to contain the output.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `AE`.
    /// 2) If the function is called before [init](AE::init) or has been finalized.
    /// 3) If the required payload length is known but has not been provided.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    // both dest and tag are updated with different size
    pub fn encrypt_final(
        &self,
        src: &[u8],
        dest: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(usize, usize)> {
        let mut dest_size: usize = dest.len();
        let mut tag_size: usize = tag.len();
        match unsafe {
            raw::TEE_AEEncryptFinal(
                self.handle(),
                src.as_ptr() as _,
                src.len(),
                dest.as_mut_ptr() as _,
                &mut dest_size,
                tag.as_mut_ptr() as _,
                &mut tag_size,
            )
        } {
            raw::TEE_SUCCESS => {
                Ok((dest_size, tag_size))
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Process data that has not been processed by previous calls to [update](AE::update) as well as data supplied in `src`.
    /// It completes the AE operation and computes the tag.
    /// The buffers `src` and `dest` SHALL be either completely disjoint or equal in their starting positions.
    /// The operation may be in either initial or active state and enters initial state afterwards.
    ///
    /// # Parameters
    ///
    /// 1) `src`: Reference to final chunk of input data to be decrypted.
    /// 2) `dest`: Output buffer. Can be omitted if the output is to be discarded, e.g. because it is known to be empty.
    /// 3) `tag`: Input buffer containing the tag to compare.
    ///
    /// # Errors
    ///
    /// `ShortBuffer`: If the output buffer is not large enough to contain the output.
    /// `MacInvalid`: If the computed tag does not match the supplied tag.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `AE`.
    /// 2) If the function is called before [init](AE::init) or has been finalized.
    /// 3) If the required payload length is known but has not been provided.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn decrypt_final(&self, src: &[u8], dest: &mut [u8], tag: &[u8]) -> Result<usize> {
        let mut dest_size: usize = dest.len();
        match unsafe {
            raw::TEE_AEDecryptFinal(
                self.handle(),
                src.as_ptr() as _,
                src.len(),
                dest.as_mut_ptr() as _,
                &mut dest_size,
                tag.as_ptr() as _,
                tag.len(),
            )
        } {
            raw::TEE_SUCCESS => {
                Ok(dest_size)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create an AE operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Function usage is similar to [Digest::allocate](Digest::allocate).
    pub fn allocate(algo: AlgorithmId, mode: OperationMode, max_key_size: usize) -> Result<Self> {
        match OperationHandle::allocate(algo, mode, max_key_size) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Function usage is similar to [Digest::info](Digest::info).
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Function usage is similar to [Digest::info_multiple](Digest::info_multiple).
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Function usage is similar to [Digest::reset](Digest::reset).
    pub fn reset(&mut self) {
        self.0.reset()
    }

    /// Function usage is similar to [Cipher::set_key](Cipher::set_key).
    pub fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        self.0.set_key(object)
    }

    /// Function usage is similar to [Digest::copy](Digest::copy).
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for AE {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for conducting asymmetric encryption /decryption or asymmetric sign / verify.
/// Note that asymmetric encryption is always “single-stage”,
/// which differs from [Cipher](Cipher) which are always “multi-stage”.
pub struct Asymmetric(OperationHandle);

impl Asymmetric {
    /// Encrypt a message.
    ///
    /// # Parameters
    ///
    /// 1) `params`: Optional operation parameters.
    /// 2) `src`: Input plaintext buffer.
    ///
    /// # Example
    /// ``` rust,no_run
    /// # use optee_utee::{TransientObject, TransientObjectType, Asymmetric};
    /// # use optee_utee::{AlgorithmId, OperationMode};
    /// # fn main() -> optee_utee::Result<()> {
    /// let clear = [1u8; 8];
    /// match TransientObject::allocate(TransientObjectType::RsaKeypair, 256) {
    ///     Ok(key) => {
    ///         key.generate_key(256, &[])?;
    ///         match Asymmetric::allocate(
    ///             AlgorithmId::RsaesPkcs1V15,
    ///             OperationMode::Encrypt,
    ///             256) {
    ///             Ok(operation) => {
    ///                 operation.set_key(&key)?;
    ///                 match operation.encrypt(&[], &clear) {
    ///                     Ok(ciph_text) => {
    ///                         // Get cipher text as a vector
    ///                         // ...
    ///                         Ok(())
    ///                     }
    ///                     Err(e) => Err(e),
    ///                 }
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
    /// 1) `ShortBuffer`: If the output buffer is not large enough to hold the result.
    /// 2) `BadParameters`: If the length of the input buffer is not consistent with the algorithm or key size.
    /// 3) `CiphertextInvalid`: If there is an error in the packing used on the ciphertext.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for [Encrypt](OperationMode::Encrypt] of
    ///    `Asymmetric`.
    /// 2) If no key is programmed in the operation.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    // This function can update output size with short buffer error when buffer is too
    // short, and example acipher utilizes this feature!
    // Define this function as unsafe because we need to return Ok for short buffer error.
    pub fn encrypt(&self, params: &[Attribute], src: &[u8]) -> Result<Vec<u8>> {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        let mut res_size: usize = self.info().key_size() as usize;
        let mut res_vec: Vec<u8> = vec![0u8; res_size];
        match unsafe {
            raw::TEE_AsymmetricEncrypt(
                self.handle(),
                p.as_ptr() as _,
                params.len() as u32,
                src.as_ptr() as _,
                src.len(),
                res_vec.as_mut_ptr() as _,
                &mut res_size,
            )
        } {
            raw::TEE_SUCCESS => {
                res_vec.truncate(res_size);
                Ok(res_vec)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Decrypt a message.
    ///
    /// # Parameters
    ///
    /// 1) `params`: Optional operation parameters.
    /// 2) `src`: Input ciphertext buffer.
    ///
    /// # Errors
    ///
    /// 1) `ShortBuffer`: If the output buffer is not large enough to hold the result.
    /// 2) `BadParameters`: If the length of the input buffer is not consistent with the algorithm or key size.
    /// 3) `CiphertextInvalid`: If there is an error in the packing used on the ciphertext.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for [Decrypt](OperationMode::Decrypt] of
    ///    `Asymmetric`.
    /// 2) If no key is programmed in the operation.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn decrypt(&self, params: &[Attribute], src: &[u8]) -> Result<Vec<u8>> {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        let mut res_size: usize = self.info().key_size() as usize;
        let mut res_vec: Vec<u8> = vec![0u8; res_size];
        match unsafe {
            raw::TEE_AsymmetricDecrypt(
                self.handle(),
                p.as_ptr() as _,
                params.len() as u32,
                src.as_ptr() as _,
                src.len(),
                res_vec.as_mut_ptr() as _,
                &mut res_size,
            )
        } {
            raw::TEE_SUCCESS => {
                res_vec.truncate(res_size);
                Ok(res_vec)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Sign a message digest.
    ///
    /// # Parameters
    ///
    /// 1) `params`: Optional operation parameters.
    /// 2) `digest`: Input buffer containing the input message digest.
    /// 3) `signature`: Output buffer written with the signature of the digest.
    ///
    /// # Errors
    ///
    /// 1) `ShortBuffer`: If `signature` is not large enough to hold the result.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for [Sign](OperationMode::Sign] of
    ///    `Asymmetric`.
    /// 2) If no key is programmed in the operation.
    /// 3) If the mode is not set as [Sign](OperationMode::Sign].
    /// 4) If `digest.len()` is not equal to the hash size of the algorithm.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn sign_digest(
        &self,
        params: &[Attribute],
        digest: &[u8],
        signature: &mut [u8],
    ) -> Result<usize> {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        let mut signature_size: usize = signature.len();
        match unsafe {
            raw::TEE_AsymmetricSignDigest(
                self.handle(),
                p.as_ptr() as _,
                params.len() as u32,
                digest.as_ptr() as _,
                digest.len(),
                signature.as_mut_ptr() as _,
                &mut signature_size,
            )
        } {
            raw::TEE_SUCCESS => {
                Ok(signature_size)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Verify a message digest.
    ///
    /// # Parameters
    ///
    /// 1) `params`: Optional operation parameters.
    /// 2) `digest`: Input buffer containing the input message digest.
    /// 3) `signature`: Input buffer containing the signature to verify.
    ///
    /// # Errors
    ///
    /// 1) `SignatureInvalid`: If the signature is invalid.
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for [Verify](OperationMode::Verify] of
    ///    `Asymmetric`.
    /// 2) If no key is programmed in the operation.
    /// 3) If the mode is not set as [Verify](OperationMode::Verify].
    /// 4) If `digest.len()` is not equal to the hash size of the algorithm.
    /// 3) Hardware or cryptographic algorithm failure.
    /// 4) If the Implementation detects any other error.
    pub fn verify_digest(
        &self,
        params: &[Attribute],
        digest: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        match unsafe {
            raw::TEE_AsymmetricVerifyDigest(
                self.handle(),
                p.as_ptr() as _,
                params.len() as u32,
                digest.as_ptr() as _,
                digest.len(),
                signature.as_ptr() as _,
                signature.len(),
            )
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Create an Asymmetric operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Function usage is similar to [Digest::allocate](Digest::allocate).
    pub fn allocate(algo: AlgorithmId, mode: OperationMode, max_key_size: usize) -> Result<Self> {
        match OperationHandle::allocate(algo, mode, max_key_size) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Function usage is similar to [Digest::info](Digest::info).
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Function usage is similar to [Digest::info_multiple](Digest::info_multiple).
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Function usage is similar to [Cipher::set_key](Cipher::set_key).
    pub fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        self.0.set_key(object)
    }

    /// Function usage is similar to [Digest::copy](Digest::copy).
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for Asymmetric {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for derive a shared key object.
pub struct DeriveKey(OperationHandle);

impl DeriveKey {
    /// Take one of the Asymmetric Derivation Operation Algorithm that supports this operation as
    /// defined in [AlgorithmId](AlgorithmId), and output a key object.
    ///
    /// # Parameters
    ///
    /// 1) `params`: For algorithm [DhDeriveSharedSecret][AlgorithmId::DhDeriveSharedSecret],
    ///    [DhPublicValue](../object/enum.AttributeId.html#variant.DhPublicValue) is required as
    ///    the passed in attribute.
    /// 2) `object`: An uninitialized transient object to be filled with the derived key.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{AttributeMemref, AttributeId, TransientObject, TransientObjectType};
    /// # use optee_utee::{DeriveKey, AlgorithmId, GenericObject};
    /// # fn example1() -> optee_utee::Result<()> {
    ///
    /// let attr_prime = AttributeMemref::from_ref(AttributeId::DhPrime, &[23u8]);
    /// let attr_base = AttributeMemref::from_ref(AttributeId::DhBase, &[5u8]);
    /// let mut public_1 = [0u8; 32];
    /// match TransientObject::allocate(TransientObjectType::DhKeypair, 256) {
    ///     Ok(key_pair_1) => {
    ///         key_pair_1.generate_key(256, &[attr_prime.into(), attr_base.into()])?;
    ///         key_pair_1.ref_attribute(AttributeId::DhPublicValue, &mut public_1)?;
    ///         Ok(())
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    ///
    /// # fn example2() -> optee_utee::Result<()> {
    /// # let mut public_1 = [0u8; 32];
    ///
    /// let attr_prime = AttributeMemref::from_ref(AttributeId::DhPrime, &[23u8]);
    /// let attr_base = AttributeMemref::from_ref(AttributeId::DhBase, &[5u8]);
    /// match TransientObject::allocate(TransientObjectType::DhKeypair, 256) {
    ///     Ok(key_pair_2) => {
    ///         key_pair_2.generate_key(256, &[attr_prime.into(), attr_base.into()])?;
    ///         match DeriveKey::allocate(AlgorithmId::DhDeriveSharedSecret, 256) {
    ///             Ok(operation) => {
    ///                 operation.set_key(&key_pair_2)?;
    ///                 match TransientObject::allocate(TransientObjectType::GenericSecret, 256) {
    ///                     // Derived key is saved as an transient object
    ///                     Ok(mut derived_key) => {
    ///                         let attr_public = AttributeMemref::from_ref(AttributeId::DhPublicValue, &public_1);
    ///                         operation.derive(&[attr_public.into()], &mut derived_key);
    ///                         // ...
    ///                         Ok(())
    ///                     }
    ///                     Err(e) => Err(e),
    ///                 }
    ///             }
    ///             Err(e) => Err(e),
    ///         }
    ///     }
    ///     Err(e) => Err(e),
    /// }
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// 1) If the algorithm is not a valid algorithm for `DeriveKey`.
    /// 2) If the `object` is too small for generated value.
    /// 3) If no key is programmed in the operation.
    /// 4) Hardware or cryptographic algorithm failure.
    /// 5) If the Implementation detects any other error.
    pub fn derive(&self, params: &[Attribute], object: &mut TransientObject) {
        let p: Vec<raw::TEE_Attribute> = params.iter().map(|p| p.raw()).collect();
        unsafe {
            raw::TEE_DeriveKey(
                self.handle(),
                p.as_ptr() as _,
                params.len() as u32,
                object.handle(),
            )
        };
    }

    /// Create a DeriveKey operation without any specific algorithm or other data.
    pub fn null() -> Self {
        Self(OperationHandle::null())
    }

    /// Function usage is similar to [Digest::allocate](Digest::allocate).
    /// Currently only supports [DhDeriveSharedSecret][AlgorithmId::DhDeriveSharedSecret] as
    /// `algo`.
    pub fn allocate(algo: AlgorithmId, max_key_size: usize) -> Result<Self> {
        match OperationHandle::allocate(algo, OperationMode::Derive, max_key_size) {
            Ok(handle) => Ok(Self(handle)),
            Err(e) => Err(e),
        }
    }

    /// Function usage is similar to [Digest::info](Digest::info).
    pub fn info(&self) -> OperationInfo {
        self.0.info()
    }

    /// Function usage is similar to [Digest::info_multiple](Digest::info_multiple).
    pub fn info_multiple(&self, info_buf: &mut [u8]) -> Result<OperationInfoMultiple> {
        self.0.info_multiple(info_buf)
    }

    /// Function usage is similar to [Cipher::set_key](Cipher::set_key).
    pub fn set_key<T: GenericObject>(&self, object: &T) -> Result<()> {
        self.0.set_key(object)
    }

    /// Function usage is similar to [Digest::copy](Digest::copy).
    pub fn copy<T: OpHandle>(&mut self, src: &T) {
        self.0.copy(src)
    }
}

impl OpHandle for DeriveKey {
    fn handle(&self) -> raw::TEE_OperationHandle {
        self.0.handle()
    }
}

/// An operation for generating random data.
pub struct Random();

impl Random {
    /// Generate random data.
    ///
    /// # Parameters
    ///
    /// 1) `res_buffer`: Reference to generated random data
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::Random;
    /// let mut res = [0u8;16];
    /// Random::generate(&mut res);
    /// ```
    ///
    /// # Panics
    ///
    /// 1) Hardware or cryptographic algorithm failure.
    /// 2) If the Implementation detects any other error.
    pub fn generate(res_buffer: &mut [u8]) {
        unsafe {
            raw::TEE_GenerateRandom(res_buffer.as_mut_ptr() as _, res_buffer.len() as _);
        }
    }
}

/// Algorithms that can be allocated as an crypto operation.
#[repr(u32)]
pub enum AlgorithmId {
    /// [Cipher](Cipher) supported algorithm.
    AesEcbNopad = 0x10000010,
    /// [Cipher](Cipher) supported algorithm.
    AesCbcNopad = 0x10000110,
    /// [Cipher](Cipher) supported algorithm.
    AesCtr = 0x10000210,
    /// [Cipher](Cipher) supported algorithm.
    AesCts = 0x10000310,
    /// [Cipher](Cipher) supported algorithm.
    AesXts = 0x10000410,
    /// [Mac](Mac) supported algorithm.
    AesCbcMacNopad = 0x30000110,
    /// [Mac](Mac) supported algorithm.
    AesCbcMacPkcs5 = 0x30000510,
    /// [Mac](Mac) supported algorithm.
    AesCmac = 0x30000610,
    /// [AE](AE) supported algorithm.
    AesCcm = 0x40000710,
    /// [AE](AE) supported algorithm.
    AesGcm = 0x40000810,
    /// [Cipher](Cipher) supported algorithm.
    DesEcbNopad = 0x10000011,
    /// [Cipher](Cipher) supported algorithm.
    DesCbcNopad = 0x10000111,
    /// [Mac](Mac) supported algorithm.
    DesCbcMacNopad = 0x30000111,
    /// [Mac](Mac) supported algorithm.
    DesCbcMacPkcs5 = 0x30000511,
    /// [Cipher](Cipher) supported algorithm.
    Des3EcbNopad = 0x10000013,
    /// [Cipher](Cipher) supported algorithm.
    Des3CbcNopad = 0x10000113,
    /// [Mac](Mac) supported algorithm.
    Des3CbcMacNopad = 0x30000113,
    /// [Mac](Mac) supported algorithm.
    Des3CbcMacPkcs5 = 0x30000513,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15 = 0xF0000830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15MD5 = 0x70001830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15Sha1 = 0x70002830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15Sha224 = 0x70003830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15Sha256 = 0x70004830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15Sha384 = 0x70005830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15Sha512 = 0x70006830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1V15MD5Sha1 = 0x7000F830,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1MD5 = 0xF0111930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1Sha1 = 0x70212930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1Sha224 = 0x70313930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1Sha256 = 0x70414930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1Sha384 = 0x70515930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    RsassaPkcs1PssMgf1Sha512 = 0x70616930,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1V15 = 0x60000130,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1MD5 = 0xF0110230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1Sha1 = 0x60210230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1Sha224 = 0x60310230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1Sha256 = 0x60410230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1Sha384 = 0x60510230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaesPkcs1OAepMgf1Sha512 = 0x60610230,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Encrypt](OperationMode::Encrypt) or [Decrypt](OperationMode::Decrypt) mode.
    RsaNopad = 0x60000030,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    DSASha1 = 0x70002131,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    DSASha224 = 0x70003131,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    DSASha256 = 0x70004131,
    /// [DeriveKey](DeriveKey) supported algorithm.
    DhDeriveSharedSecret = 0x80000032,
    /// [DeriveKey](DeriveKey) supported algorithm.
    EcDhDeriveSharedSecret = 0x80000042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    EcDsaSha1 = 0x70001042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    EcDsaSha224 = 0x70002042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    EcDsaSha256 = 0x70003042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    EcDsaSha384 = 0x70004042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    EcDsaSha512 = 0x70005042,
    /// [Asymmetric](Asymmetric) supported algorithm, can be applied with
    /// [Sign](OperationMode::Sign) or [Verify](OperationMode::Verify) mode.
    Ed25519 = 0x70006043,
    /// [DeriveKey](DeriveKey) supported algorithm.
    X25519 = 0x80000044,
    /// [Digest](Digest) supported algorithm.
    Md5 = 0x50000001,
    /// [Digest](Digest) supported algorithm.
    Sha1 = 0x50000002,
    /// [Digest](Digest) supported algorithm.
    Sha224 = 0x50000003,
    /// [Digest](Digest) supported algorithm.
    Sha256 = 0x50000004,
    /// [Digest](Digest) supported algorithm.
    Sha384 = 0x50000005,
    /// [Digest](Digest) supported algorithm.
    Sha512 = 0x50000006,
    /// [Mac](Mac) supported algorithm.
    Md5Sha1 = 0x5000000F,
    /// [Mac](Mac) supported algorithm.
    HmacMd5 = 0x30000001,
    /// [Mac](Mac) supported algorithm.
    HmacSha1 = 0x30000002,
    /// [Mac](Mac) supported algorithm.
    HmacSha224 = 0x30000003,
    /// [Mac](Mac) supported algorithm.
    HmacSha256 = 0x30000004,
    /// [Mac](Mac) supported algorithm.
    HmacSha384 = 0x30000005,
    /// [Mac](Mac) supported algorithm.
    HmacSha512 = 0x30000006,
    /// Reserved for GlobalPlatform compliance test applications.
    IllegalValue = 0xefffffff,
}

/// This specification defines support for optional cryptographic elements.
#[repr(u32)]
pub enum ElementId {
    /// Where algId fully defines the required support,
    /// the special value TEE_CRYPTO_ELEMENT_NONE should be used
    ElementNone = 0x00000000,
    /// Source: `NIST`, Generic: `Y`, Size: 192 bits
    EccCurveNistP192 = 0x00000001,
    /// Source: `NIST`, Generic: `Y`, Size: 224 bits
    EccCurveNistP224 = 0x00000002,
    /// Source: `NIST`, Generic: `Y`, Size: 256 bits
    EccCurveNistP256 = 0x00000003,
    /// Source: `NIST`, Generic: `Y`, Size: 384 bits
    EccCurveNistP384 = 0x00000004,
    /// Source: `NIST`, Generic: `Y`, Size: 521 bits
    EccCurveNistP521 = 0x00000005,
    /// Source: `IETF`, Generic: `N`, Size: 256 bits
    EccCurve25519 = 0x00000300,
}
