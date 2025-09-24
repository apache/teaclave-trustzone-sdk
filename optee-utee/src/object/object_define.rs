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
            Whence::DataSeekSet => raw::TEE_Whence::TEE_DATA_SEEK_SET,
            Whence::DataSeekCur => raw::TEE_Whence::TEE_DATA_SEEK_CUR,
            Whence::DataSeekEnd => raw::TEE_Whence::TEE_DATA_SEEK_END,
        }
    }
}

#[repr(u32)]
pub enum ObjectStorageConstants {
    Private = 0x00000001,
    IllegalValue = 0x7FFFFFFF,
}

bitflags! {
    /// A set of flags that controls the access rights and sharing permissions
    /// with which the object handle is opened.
    pub struct DataFlag: u32 {
        /// The object is opened with the read access right. This allows the
        /// Trusted Application to call the function `TEE_ReadObjectData`.
        const ACCESS_READ = 0x00000001;
        /// The object is opened with the write access right. This allows the
        /// Trusted Application to call the functions `TEE_WriteObjectData` and
        /// `TEE_TruncateObjectData`.
        const ACCESS_WRITE = 0x00000002;
        /// The object is opened with the write-meta access right. This allows
        /// the Trusted Application to call the functions
        /// `TEE_CloseAndDeletePersistentObject1` and
        /// `TEE_RenamePersistentObject`.
        const ACCESS_WRITE_META = 0x00000004;
        /// The caller allows another handle on the object to be created with
        /// read access.
        const SHARE_READ = 0x00000010;
        /// The caller allows another handle on the object to be created with
        /// write access.
        const SHARE_WRITE = 0x00000020;
        /// * If this flag is present and the object exists, then the object is
        ///   deleted and re-created as an atomic operation: that is, the TA
        ///   sees either the old object or the new one.
        /// * If the flag is absent and the object exists, then the function
        ///   SHALL return `TEE_ERROR_ACCESS_CONFLICT`.
        const OVERWRITE = 0x00000400;
    }
}

bitflags! {
    /// A set of flags that defines usages of data in TEE secure storage.
    pub struct UsageFlag: u32 {
        /// The object [Attribute](Attribute) can be extracted.
        const EXTRACTABLE = 0x00000001;
        /// The object can be used for encryption.
        const ENCRYPT = 0x00000002;
        /// The object can be used for decryption.
        const DECRYPT = 0x00000004;
        /// The object can be used for mac operation.
        const MAC = 0x00000008;
        /// The object can be used for signature.
        const SIGN = 0x00000010;
        /// The object can be used for verification of a signature.
        const VERIFY = 0x00000020;
        /// The object can be used for deriving a key.
        const DERIVE = 0x00000040;
    }
}

/// Miscellaneous constants.
#[repr(u32)]
pub enum MiscellaneousConstants {
    /// Maximum offset of a data object.
    TeeDataMaxPosition = 0xFFFFFFFF,
    /// Maximum length of an object id.
    TeeObjectIdMaxLen = 64,
}

bitflags! {
    /// A set of flags that defines Handle features.
    pub struct HandleFlag: u32{
        /// Set for a [PersistentObject](crate::PersistentObject).
        const PERSISTENT = 0x00010000;
        /// 1) For a [PersistentObject](crate::PersistentObject), always set.
        /// 2) For a [TransientObject](crate::TransientObject), initially
        ///    cleared, then set when the object becomes initialized.
        const INITIALIZED = 0x00020000;
        /// Following two flags are for crypto operation handles:
        /// 1) Set if the required operation key has been set.
        /// 2) Always set for digest operations.
        const KEY_SET = 0x00040000;
        /// Set if the algorithm expects two keys to be set, using
        /// `TEE_SetOperationKey2`.
        /// This happens only if algorithm is set to
        /// [AesXts](crate::AlgorithmId::AesXts)
        /// or `TEE_ALG_SM2_KEP`(not supported now).
        const EXPECT_TWO_KEYS = 0x00080000;
    }
}
