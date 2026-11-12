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

use crate::{Error, ErrorKind, Result, tee_check};
use crate::{Identity, Uuid};
use alloc::{string::String, vec::Vec};
use core::ffi::CStr;
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
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self>;
}

/// Implements the PropertyValue trait for all return types:
/// String, Bool, u32, u64, BinaryBlock, UUID, Identity.
impl PropertyValue for String {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut out_size = 0;

        // The first call is to get the size of the string
        // So we pass a null pointer and a size of 0
        let res = unsafe {
            raw::TEE_GetPropertyAsString(set, key.as_ptr(), core::ptr::null_mut(), &mut out_size)
        };
        match res {
            raw::TEE_SUCCESS => {
                if out_size == 0 {
                    // return an empty string
                    Ok(String::new())
                } else {
                    Err(Error::new(ErrorKind::Generic))
                }
            }
            raw::TEE_ERROR_SHORT_BUFFER => {
                // Resize the string to the actual size
                let mut out_buffer = vec![0; out_size];
                let res = unsafe {
                    raw::TEE_GetPropertyAsString(
                        set,
                        key.as_ptr(),
                        out_buffer.as_mut_ptr() as *mut _,
                        &mut out_size,
                    )
                };
                tee_check(res)?;

                // Convert the char buffer with null terminator to a C string
                let c_str = CStr::from_bytes_with_nul(&out_buffer)
                    .map_err(|_| Error::new(ErrorKind::BadFormat))?;
                // Convert the C string to a Rust string
                let result = c_str.to_string_lossy().into_owned();

                Ok(result)
            }
            _ => Err(Error::from_raw_error(res)),
        }
    }
}

impl PropertyValue for bool {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut b: bool = false;

        let res = unsafe { raw::TEE_GetPropertyAsBool(set, key.as_ptr(), &mut b) };
        tee_check(res)?;

        Ok(b)
    }
}

impl PropertyValue for u32 {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut value = 0;

        let res = unsafe { raw::TEE_GetPropertyAsU32(set, key.as_ptr(), &mut value) };
        tee_check(res)?;

        Ok(value)
    }
}

impl PropertyValue for u64 {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut value = 0;

        let res = unsafe { raw::TEE_GetPropertyAsU64(set, key.as_ptr(), &mut value) };
        tee_check(res)?;

        Ok(value)
    }
}

impl PropertyValue for Vec<u8> {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut out_size = 0;

        // The first call is to get the size of the binary block
        // So we pass a null pointer and a size of 0
        let res = unsafe {
            raw::TEE_GetPropertyAsBinaryBlock(
                set,
                key.as_ptr(),
                core::ptr::null_mut(),
                &mut out_size,
            )
        };

        match res {
            raw::TEE_SUCCESS => {
                if out_size == 0 {
                    // return an empty buffer
                    Ok(vec![])
                } else {
                    Err(Error::new(ErrorKind::Generic))
                }
            }
            raw::TEE_ERROR_SHORT_BUFFER => {
                let mut buf = vec![0; out_size];

                let res = unsafe {
                    raw::TEE_GetPropertyAsBinaryBlock(
                        set,
                        key.as_ptr(),
                        buf.as_mut_ptr() as *mut _,
                        &mut out_size,
                    )
                };
                tee_check(res)?;
                Ok(buf)
            }
            _ => Err(Error::from_raw_error(res)),
        }
    }
}

impl PropertyValue for Uuid {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
        let mut raw_uuid = raw::TEE_UUID {
            timeLow: 0,
            timeMid: 0,
            timeHiAndVersion: 0,
            clockSeqAndNode: [0; 8],
        };

        let res = unsafe { raw::TEE_GetPropertyAsUUID(set, key.as_ptr(), &mut raw_uuid) };
        tee_check(res)?;

        Ok(Uuid::from(raw_uuid))
    }
}

impl PropertyValue for Identity {
    unsafe fn from_raw(set: raw::TEE_PropSetHandle, key: &CStr) -> Result<Self> {
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

        let res = unsafe { raw::TEE_GetPropertyAsIdentity(set, key.as_ptr(), &mut raw_id) };
        tee_check(res)?;

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
    /// The property name, as a NUL-terminated C string.
    fn key(&self) -> &'static CStr;
    fn set(&self) -> PropertySet;

    fn get(&self) -> Result<Self::Output> {
        unsafe { Self::Output::from_raw(self.set().as_raw(), self.key()) }
    }
}

/// Macro to define a property key.
///
/// This macro generates a struct that implements the [`PropertyKey`] trait.
///
/// The key can be given in two forms:
///
/// * a string literal, to which the NUL terminator expected by the C API is
///   appended (`"gpd.ta.appID"`);
/// * a NUL-terminated byte-string constant, such as the `TA_PROP_STR_*` items
///   exported by `optee_utee_sys` (`raw::TA_PROP_STR_SINGLE_INSTANCE`). These
///   are generated from the OP-TEE headers, so prefer them over repeating the
///   string by hand.
///
/// Keys are compile-time constants. `CStr::from_bytes_with_nul` is a `const`
/// function, so the conversion happens during const evaluation and a key that
/// is not NUL-terminated or has an interior NUL fails to compile.
///
/// ```
/// use optee_utee::define_property_key;
/// use optee_utee::property::PropertyKey;
/// use optee_utee::Uuid;
/// use optee_utee_sys as raw;
///
/// define_property_key!(MyAppId, CurrentTa, "gpd.ta.appID", Uuid);
/// define_property_key!(
///     MySingleInstance,
///     CurrentTa,
///     raw::TA_PROP_STR_SINGLE_INSTANCE,
///     bool
/// );
///
/// fn main() {
///     assert_eq!(MyAppId.key().to_bytes(), b"gpd.ta.appID");
///     assert_eq!(
///         MySingleInstance.key().to_bytes(),
///         b"gpd.ta.singleInstance"
///     );
/// }
/// ```
///
/// A key with an interior NUL byte is rejected at compile time:
///
/// ```compile_fail,E0080
/// use optee_utee::define_property_key;
///
/// define_property_key!(BadKey, CurrentTa, "gpd\0ta.appID", u32);
///
/// fn main() {}
/// ```
///
/// So is a constant that is not NUL-terminated:
///
/// ```compile_fail,E0080
/// use optee_utee::define_property_key;
///
/// const NOT_TERMINATED: &[u8; 3] = b"abc";
/// define_property_key!(BadKey, CurrentTa, NOT_TERMINATED, u32);
///
/// fn main() {}
/// ```
#[macro_export]
macro_rules! define_property_key {
    ($name:ident, $set:ident, $key:literal, $output:ty) => {
        // `concat!` appends the terminator without allocating.
        $crate::define_property_key!(@key $name, $set, $output, concat!($key, "\0").as_bytes());
    };
    ($name:ident, $set:ident, $key:expr, $output:ty) => {
        // The generated `TA_PROP_STR_*` constants already include it.
        $crate::define_property_key!(@key $name, $set, $output, $key);
    };
    (@key $name:ident, $set:ident, $output:ty, $bytes:expr) => {
        pub struct $name;

        impl $crate::property::PropertyKey for $name {
            type Output = $output;

            fn key(&self) -> &'static ::core::ffi::CStr {
                const KEY: &::core::ffi::CStr =
                    match ::core::ffi::CStr::from_bytes_with_nul($bytes) {
                        Ok(key) => key,
                        Err(_) => panic!(
                            "property key must be NUL-terminated without interior NUL bytes"
                        ),
                    };
                KEY
            }

            fn set(&self) -> $crate::property::PropertySet {
                $crate::property::PropertySet::$set
            }
        }
    };
}

// Define all existing property keys for the TEE property set.
// The format is:
// `define_property_key!(Name, Set, Key, OutputType);`
// The `Set` is one of the PropertySet it belongs to.
// The `Key` is either a string literal or a NUL-terminated `TA_PROP_STR_*`
// constant from `optee_utee_sys`; prefer the latter, as it is generated from
// the OP-TEE headers. The remaining `gpd.*` names have no macro counterpart:
// OP-TEE only spells them out as string literals in `core/tee/tee_svc.c` and
// `lib/libutee/tee_api_property.c`, which bindgen does not read. Only the
// properties those files implement are listed here; callers that need another
// name can define their own key with `define_property_key!`.
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
    raw::TA_PROP_STR_SINGLE_INSTANCE,
    bool
);
define_property_key!(
    TaMultiSession,
    CurrentTa,
    raw::TA_PROP_STR_MULTI_SESSION,
    bool
);
define_property_key!(
    TaInstanceKeepAlive,
    CurrentTa,
    raw::TA_PROP_STR_KEEP_ALIVE,
    bool
);
define_property_key!(
    TaInstanceKeepCrashed,
    CurrentTa,
    raw::TA_PROP_STR_KEEP_CRASHED,
    bool
);
define_property_key!(TaDataSize, CurrentTa, raw::TA_PROP_STR_DATA_SIZE, u32);
define_property_key!(TaStackSize, CurrentTa, raw::TA_PROP_STR_STACK_SIZE, u32);
define_property_key!(TaVersion, CurrentTa, raw::TA_PROP_STR_VERSION, String);
define_property_key!(
    TaDescription,
    CurrentTa,
    raw::TA_PROP_STR_DESCRIPTION,
    String
);
define_property_key!(TaEndian, CurrentTa, raw::TA_PROP_STR_ENDIAN, u32);
define_property_key!(
    TaDoesNotCloseHandleOnCorruptObject,
    CurrentTa,
    raw::TA_PROP_STR_DOES_NOT_CLOSE_HANDLE_ON_CORRUPT_OBJECT,
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
define_property_key!(TeeDeviceId, TeeImplementation, "gpd.tee.deviceID", Uuid);
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
    TeeSocketsVersion,
    TeeImplementation,
    "gpd.tee.sockets.version",
    u32
);
define_property_key!(
    TeeSocketsTcpVersion,
    TeeImplementation,
    "gpd.tee.sockets.tcp.version",
    u32
);
define_property_key!(
    TeeCryptographyEcc,
    TeeImplementation,
    "gpd.tee.cryptography.ecc",
    bool
);
define_property_key!(
    TeeTrustedStorageAntiRollbackProtectionLevel,
    TeeImplementation,
    "gpd.tee.trustedStorage.antiRollback.protectionLevel",
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
    u32
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
    u32
);
define_property_key!(
    TeeFirmwareManufacturer,
    TeeImplementation,
    "gpd.tee.firmware.manufacturer",
    String
);
