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

use crate::{Error, ErrorKind, Result};
use crate::{Identity, Uuid};
use alloc::{ffi::CString, string::String, vec::Vec};
use optee_utee_sys as raw;

/// Represents a TEE property set according to the TEE Internal API.
/// The property set is a collection of properties that can be
/// queried from the TEE. The property set is identified by a
/// handle, which is a pointer to a TEE_PropSetHandle structure.
pub enum PropertySet {
    TeeImplementation,
    CurrentClient,
    CurrentTa,
}

impl PropertySet {
    fn as_raw(&self) -> raw::TEE_PropSetHandle {
        match self {
            PropertySet::TeeImplementation => raw::TEE_PROPSET_TEE_IMPLEMENTATION,
            PropertySet::CurrentClient => raw::TEE_PROPSET_CURRENT_CLIENT,
            PropertySet::CurrentTa => raw::TEE_PROPSET_CURRENT_TA,
        }
    }
}

/// Represents a TEE property value.
/// The property value can be of different types, such as
/// string, bool, u32, TEE_UUID, TEE_Identity, etc.
/// The property value is obtained from the TEE
/// property set using the TEE_GetPropertyAs* functions.
pub trait PropertyValue: Sized {
    /// # Safety
    /// This function is unsafe because it dereferences raw pointers from the TEE API.
    /// The caller must ensure that the `set` handle is valid and that the TEE environment
    /// is properly initialized.
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self>;
}

/// Implements the PropertyValue trait for all return types:
/// String, Bool, u32, u64, BinaryBlock, UUID, Identity.
impl PropertyValue for String {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut out_size = 0;

        // The first call is to get the size of the string
        // So we pass a null pointer and a size of 0
        let res = {
            raw::TEE_GetPropertyAsString(
                set,
                key.as_ptr() as *const core::ffi::c_char,
                core::ptr::null_mut(),
                &mut out_size,
            )
        };
        match res {
            raw::TEE_SUCCESS => {
                if out_size == 0 {
                    // return an empty string
                    Ok(String::new())
                }
                else {
                    Err(Error::new(ErrorKind::Generic))
                }
            }
            raw::TEE_ERROR_SHORT_BUFFER => {
                // Resize the string to the actual size
                let mut out_buffer = vec![0; out_size];
                let res = {
                    raw::TEE_GetPropertyAsString(
                        set,
                        key.as_ptr() as *const core::ffi::c_char,
                        out_buffer.as_mut_ptr() as *mut core::ffi::c_char,
                        &mut out_size,
                    )
                };
                if res != raw::TEE_SUCCESS {
                    return Err(Error::from_raw_error(res));
                }

                // Convert the char buffer with null terminator to a C string
                let c_str = core::ffi::CStr::from_bytes_with_nul(&out_buffer)
                    .map_err(|_| Error::new(ErrorKind::BadFormat))?;
                // Convert the C string to a Rust string
                let result = c_str.to_string_lossy().into_owned();

                Ok(result)
            }
            _ => {
                Err(Error::from_raw_error(res))
            }
        }
    }
}

impl PropertyValue for bool {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut b: bool = false;

        let res = raw::TEE_GetPropertyAsBool(set, key.as_ptr() as *const core::ffi::c_char, &mut b);
        if res != 0 {
            return Err(Error::from_raw_error(res));
        }

        Ok(b)
    }
}

impl PropertyValue for u32 {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut value = 0;

        let res = raw::TEE_GetPropertyAsU32(set, key.as_ptr() as *const core::ffi::c_char, &mut value);
        if res != 0 {
            return Err(Error::from_raw_error(res));
        }

        Ok(value)
    }
}

impl PropertyValue for u64 {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut value = 0;

        let res = raw::TEE_GetPropertyAsU64(set, key.as_ptr() as *const core::ffi::c_char, &mut value);
        if res != 0 {
            return Err(Error::from_raw_error(res));
        }

        Ok(value)
    }
}

impl PropertyValue for Vec<u8> {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut out_size = 0;

        // The first call is to get the size of the binary block
        // So we pass a null pointer and a size of 0
        let res = {
            raw::TEE_GetPropertyAsBinaryBlock(
                set,
                key.as_ptr() as *const core::ffi::c_char,
                core::ptr::null_mut(),
                &mut out_size,
            )
        };

        match res {
            raw::TEE_SUCCESS => {
                if out_size == 0 {
                    // return an empty buffer
                    Ok(vec![])
                }
                else {
                    Err(Error::new(ErrorKind::Generic))
                }
            }
            raw::TEE_ERROR_SHORT_BUFFER => {
                let mut buf = vec![0; out_size];

                let res = {
                    raw::TEE_GetPropertyAsBinaryBlock(
                        set,
                        key.as_ptr() as *const core::ffi::c_char,
                        buf.as_mut_ptr() as *mut core::ffi::c_void,
                        &mut out_size,
                    )
                };
                if res != raw::TEE_SUCCESS {
                    Err(Error::from_raw_error(res))
                } else {
                    Ok(buf)
                }
            }
            _ => {
                Err(Error::from_raw_error(res))
            }
        }
    }
}

impl PropertyValue for Uuid {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        let mut raw_uuid = raw::TEE_UUID {
            timeLow: 0,
            timeMid: 0,
            timeHiAndVersion: 0,
            clockSeqAndNode: [0; 8],
        };

        let res =
            raw::TEE_GetPropertyAsUUID(set, key.as_ptr() as *const core::ffi::c_char, &mut raw_uuid);
        if res != 0 {
            return Err(Error::from_raw_error(res));
        }

        Ok(Uuid::from(raw_uuid))
    }
}

impl PropertyValue for Identity {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: CString) -> Result<Self> {
        // Allocate a buffer for the raw identity
        let mut raw_id = raw::TEE_Identity {
            login: 0,
            uuid: raw::TEE_UUID {
                timeLow: 0,
                timeMid: 0,
                timeHiAndVersion: 0,
                clockSeqAndNode: [0; 8],
            },
        };

        let res = raw::TEE_GetPropertyAsIdentity(set, key.as_ptr() as *const core::ffi::c_char, &mut raw_id);
        if res != 0 {
            return Err(Error::from_raw_error(res));
        }

        Ok(Identity::from(raw_id))
    }
}

/// Represents a TEE property key.
/// The property key is used to identify a specific property
/// within a property set. The property key is a string that
/// is used to query the property value from the TEE property
/// set. The property key is defined in the TEE Internal API,
/// such as "gpd.client.identity" or "gpd.tee.apiversion".
pub trait PropertyKey {
    type Output: PropertyValue;
    fn key(&self) -> CString;
    fn set(&self) -> PropertySet;

    fn get(&self) -> Result<Self::Output> {
        unsafe { Self::Output::from_raw(self.set().as_raw(), self.key()) }
    }
}

/// Macro to define a property key.
/// This macro generates a struct that implements the
/// PropertyKey trait.
macro_rules! define_property_key {
    (
        $name:ident,
        $set:ident,
        $key:literal,
        $output:ty
    ) => {
        pub struct $name;

        impl PropertyKey for $name {
            type Output = $output;

            fn key(&self) -> CString {
                CString::new($key).unwrap_or_default()
            }

            fn set(&self) -> PropertySet {
                PropertySet::$set
            }
        }
    };
}

// Define all existing property keys for the TEE property set.
// The format is:
// `define_property_key!(Name, Set, "key", OutputType);`
// The `Set` is one of the PropertySet it belongs to.
// The `key` is the raw property key string.
// The `OutputType` is the type of the property value.
// 
// To get the property value, use the `get` method.
// Example usage:
// 
// ``` no_run
// use optee_utee::{PropertyKey, TaAppId};
// 
// let my_property = TaAppId.get()?;
// ```
define_property_key!(TaAppId, CurrentTa, "gpd.ta.appID", Uuid);
define_property_key!(
    TaSingleInstance,
    CurrentTa,
    "gpd.ta.singleInstance",
    bool
);
define_property_key!(
    TaMultiSession,
    CurrentTa,
    "gpd.ta.multiSession",
    bool
);
define_property_key!(
    TaInstanceKeepAlive,
    CurrentTa,
    "gpd.ta.instanceKeepAlive",
    bool
);
define_property_key!(TaDataSize, CurrentTa, "gpd.ta.dataSize", u32);
define_property_key!(TaStackSize, CurrentTa, "gpd.ta.stackSize", u32);
define_property_key!(TaVersion, CurrentTa, "gpd.ta.version", String);
define_property_key!(
    TaDescription,
    CurrentTa,
    "gpd.ta.description",
    String
);
define_property_key!(TaEndian, CurrentTa, "gpd.ta.endian", u32);
define_property_key!(
    TaDoesNotCloseHandleOnCorruptObject,
    CurrentTa,
    "gpd.ta.doesNotCloseHandleOnCorruptObject",
    bool
);
define_property_key!(
    ClientIdentity,
    CurrentClient,
    "gpd.client.identity",
    Identity
);
define_property_key!(ClientEndian, CurrentClient, "gpd.client.endian", u32);
define_property_key!(
    TeeApiVersion,
    TeeImplementation,
    "gpd.tee.apiversion",
    String
);
define_property_key!(
    TeeInternalCoreVersion,
    TeeImplementation,
    "gpd.tee.internalCore.version",
    u32
);
define_property_key!(
    TeeDescription,
    TeeImplementation,
    "gpd.tee.description",
    String
);
define_property_key!(
    TeeDeviceId,
    TeeImplementation,
    "gpd.tee.deviceID",
    Uuid
);
define_property_key!(
    TeeSystemTimeProtectionLevel,
    TeeImplementation,
    "gpd.tee.systemTime.protectionLevel",
    u32
);
define_property_key!(
    TeeTaPersistentTimeProtectionLevel,
    TeeImplementation,
    "gpd.tee.TAPersistentTime.protectionLevel",
    u32
);
define_property_key!(
    TeeArithMaxBigIntSize,
    TeeImplementation,
    "gpd.tee.arith.maxBigIntSize",
    u32
);
define_property_key!(
    TeeCryptographyEcc,
    TeeImplementation,
    "gpd.tee.cryptography.ecc",
    bool
);
define_property_key!(
    TeeCryptographyNist,
    TeeImplementation,
    "gpd.tee.cryptography.nist",
    bool
);
define_property_key!(
    TeeCryptographyBsiR,
    TeeImplementation,
    "gpd.tee.cryptography.bsi-r",
    bool
);
define_property_key!(
    TeeCryptographyBsiT,
    TeeImplementation,
    "gpd.tee.cryptography.bsi-t",
    bool
);
define_property_key!(
    TeeCryptographyIetf,
    TeeImplementation,
    "gpd.tee.cryptography.ietf",
    bool
);
define_property_key!(
    TeeCryptographyOcta,
    TeeImplementation,
    "gpd.tee.cryptography.octa",
    bool
);
define_property_key!(
    TeeTrustedStoragePrivateRollbackProtection,
    TeeImplementation,
    "gpd.tee.trustedStorage.private.rollbackProtection",
    u32
);
define_property_key!(
    TeeTrustedStoragePersoRollbackProtection,
    TeeImplementation,
    "gpd.tee.trustedStorage.perso.rollbackProtection",
    u32
);
define_property_key!(
    TeeTrustedStorageProtectedRollbackProtection,
    TeeImplementation,
    "gpd.tee.trustedStorage.protected.rollbackProtection",
    u32
);
define_property_key!(
    TeeTrustedStorageAntiRollbackProtectionLevel,
    TeeImplementation,
    "gpd.tee.trustedStorage.antiRollback.protectionLevel",
    u32
);
define_property_key!(
    TeeTrustedStorageRollbackDetectionProtectionLevel,
    TeeImplementation,
    "gpd.tee.trustedStorage.rollbackDetection.protectionLevel",
    u32
);
define_property_key!(
    TeeTrustedOsImplementationVersion,
    TeeImplementation,
    "gpd.tee.trustedos.implementation.version",
    String
);
define_property_key!(
    TeeTrustedOsImplementationBinaryVersion,
    TeeImplementation,
    "gpd.tee.trustedos.implementation.binaryversion",
    Vec<u8>
);
define_property_key!(
    TeeTrustedOsManufacturer,
    TeeImplementation,
    "gpd.tee.trustedos.manufacturer",
    String
);
define_property_key!(
    TeeFirmwareImplementationVersion,
    TeeImplementation,
    "gpd.tee.firmware.implementation.version",
    String
);
define_property_key!(
    TeeFirmwareImplementationBinaryVersion,
    TeeImplementation,
    "gpd.tee.firmware.implementation.binaryversion",
    Vec<u8>
);
define_property_key!(
    TeeFirmwareManufacturer,
    TeeImplementation,
    "gpd.tee.firmware.manufacturer",
    String
);
define_property_key!(
    TeeEventMaxSources,
    TeeImplementation,
    "gpd.tee.event.maxSources",
    u32
);
