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

use bitflags::bitflags;
use optee_utee_sys as raw;

/// Indicate the possible start offset when moving a data position in the data
/// stream associated with a [PersistentObject](crate::PersistentObject).
pub enum Whence {
    /// The data position is set to offset bytes from the beginning of the data stream.
    DataSeekSet,
    /// The data position is set to its current position plus offset.
    DataSeekCur,
    /// The data position is set to the size of the object data plus offset.
    DataSeekEnd,
}

impl From<Whence> for raw::TEE_Whence {
    fn from(val: Whence) -> Self {
        match val {
            Whence::DataSeekSet => raw::TEE_DATA_SEEK_SET,
            Whence::DataSeekCur => raw::TEE_DATA_SEEK_CUR,
            Whence::DataSeekEnd => raw::TEE_DATA_SEEK_END,
        }
    }
}

#[repr(u32)]
pub enum ObjectStorageConstants {
    Private = raw::TEE_STORAGE_PRIVATE,
    PrivateRee = raw::TEE_STORAGE_PRIVATE_REE,
    PrivateRpmb = raw::TEE_STORAGE_PRIVATE_RPMB,
    /// OP-TEE has no macro for this value, so it stays a literal.
    IllegalValue = 0x7FFFFFFF,
}

bitflags! {
    /// A set of flags that controls the access rights and sharing permissions
    /// with which the object handle is opened.
    pub struct DataFlag: u32 {
        /// The object is opened with the read access right. This allows the
        /// Trusted Application to call the function `TEE_ReadObjectData`.
        const ACCESS_READ = raw::TEE_DATA_FLAG_ACCESS_READ;
        /// The object is opened with the write access right. This allows the
        /// Trusted Application to call the functions `TEE_WriteObjectData` and
        /// `TEE_TruncateObjectData`.
        const ACCESS_WRITE = raw::TEE_DATA_FLAG_ACCESS_WRITE;
        /// The object is opened with the write-meta access right. This allows
        /// the Trusted Application to call the functions
        /// `TEE_CloseAndDeletePersistentObject1` and
        /// `TEE_RenamePersistentObject`.
        const ACCESS_WRITE_META = raw::TEE_DATA_FLAG_ACCESS_WRITE_META;
        /// The caller allows another handle on the object to be created with
        /// read access.
        const SHARE_READ = raw::TEE_DATA_FLAG_SHARE_READ;
        /// The caller allows another handle on the object to be created with
        /// write access.
        const SHARE_WRITE = raw::TEE_DATA_FLAG_SHARE_WRITE;
        /// * If this flag is present and the object exists, then the object is
        ///   deleted and re-created as an atomic operation: that is, the TA
        ///   sees either the old object or the new one.
        /// * If the flag is absent and the object exists, then the function
        ///   SHALL return `TEE_ERROR_ACCESS_CONFLICT`.
        const OVERWRITE = raw::TEE_DATA_FLAG_OVERWRITE;
    }
}

bitflags! {
    /// A set of flags that defines usages of data in TEE secure storage.
    pub struct UsageFlag: u32 {
        /// The object [Attribute](Attribute) can be extracted.
        const EXTRACTABLE = raw::TEE_USAGE_EXTRACTABLE;
        /// The object can be used for encryption.
        const ENCRYPT = raw::TEE_USAGE_ENCRYPT;
        /// The object can be used for decryption.
        const DECRYPT = raw::TEE_USAGE_DECRYPT;
        /// The object can be used for mac operation.
        const MAC = raw::TEE_USAGE_MAC;
        /// The object can be used for signature.
        const SIGN = raw::TEE_USAGE_SIGN;
        /// The object can be used for verification of a signature.
        const VERIFY = raw::TEE_USAGE_VERIFY;
        /// The object can be used for deriving a key.
        const DERIVE = raw::TEE_USAGE_DERIVE;
    }
}

/// Miscellaneous constants.
#[repr(u32)]
pub enum MiscellaneousConstants {
    /// Maximum offset of a data object.
    TeeDataMaxPosition = raw::TEE_DATA_MAX_POSITION,
    /// Maximum length of an object id.
    TeeObjectIdMaxLen = raw::TEE_OBJECT_ID_MAX_LEN,
}

bitflags! {
    /// A set of flags that defines Handle features.
    pub struct HandleFlag: u32{
        /// Set for a [PersistentObject](crate::PersistentObject).
        const PERSISTENT = raw::TEE_HANDLE_FLAG_PERSISTENT;
        /// 1) For a [PersistentObject](crate::PersistentObject), always set.
        /// 2) For a [TransientObject](crate::TransientObject), initially
        ///    cleared, then set when the object becomes initialized.
        const INITIALIZED = raw::TEE_HANDLE_FLAG_INITIALIZED;
        /// Following two flags are for crypto operation handles:
        /// 1) Set if the required operation key has been set.
        /// 2) Always set for digest operations.
        const KEY_SET = raw::TEE_HANDLE_FLAG_KEY_SET;
        /// Set if the algorithm expects two keys to be set, using
        /// `TEE_SetOperationKey2`.
        /// This happens only if algorithm is set to
        /// [AesXts](crate::AlgorithmId::AesXts)
        /// or `TEE_ALG_SM2_KEP`(not supported now).
        const EXPECT_TWO_KEYS = raw::TEE_HANDLE_FLAG_EXPECT_TWO_KEYS;
    }
}
