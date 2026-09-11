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

#[cfg(not(feature = "std"))]
use core::error;
use core::{fmt, result};
use num_enum::{FromPrimitive, IntoPrimitive};
use optee_utee_sys as raw;
#[cfg(feature = "std")]
use std::error;

/// A specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for TEE operations.
///
/// # Examples
///
/// ``` rust,no_run
/// # use optee_utee::prelude::*;
/// use optee_utee::Result;
/// fn open_session(params: &mut ParametersAny) -> Result<()> {
///     Ok(())
/// }
/// ````
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
    origin: Option<ErrorOrigin>,
}

/// A list specifying general categories of TEE error and its corresponding code
/// in OP-TEE OS.
///
/// Unrecognized codes are preserved in the `Unknown` catch-all variant, so
/// `raw_code()` always round-trips the value passed to
/// [`Error::from_raw_error`].
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive,
)]
#[repr(u32)]
pub enum ErrorKind {
    /// Object corruption.
    CorruptObject = raw::TEE_ERROR_CORRUPT_OBJECT,
    /// Persistent object corruption.
    CorruptObject2 = raw::TEE_ERROR_CORRUPT_OBJECT_2,
    /// Object storage is not available.
    StorageNotAvailable = raw::TEE_ERROR_STORAGE_NOT_AVAILABLE,
    /// Persistent object storage is not available.
    StorageNotAvailable2 = raw::TEE_ERROR_STORAGE_NOT_AVAILABLE_2,
    /// Non-specific cause.
    Generic = raw::TEE_ERROR_GENERIC,
    /// Access privileges are not sufficient.
    AccessDenied = raw::TEE_ERROR_ACCESS_DENIED,
    /// The operation was canceled.
    Cancel = raw::TEE_ERROR_CANCEL,
    /// Concurrent accesses caused conflict.
    AccessConflict = raw::TEE_ERROR_ACCESS_CONFLICT,
    /// Too much data for the requested operation was passed.
    ExcessData = raw::TEE_ERROR_EXCESS_DATA,
    /// Input data was of invalid format.
    BadFormat = raw::TEE_ERROR_BAD_FORMAT,
    /// Input parameters were invalid.
    BadParameters = raw::TEE_ERROR_BAD_PARAMETERS,
    /// Operation is not valid in the current state.
    BadState = raw::TEE_ERROR_BAD_STATE,
    /// The requested data item is not found.
    ItemNotFound = raw::TEE_ERROR_ITEM_NOT_FOUND,
    /// The requested operation should exist but is not yet implemented.
    NotImplemented = raw::TEE_ERROR_NOT_IMPLEMENTED,
    /// The requested operation is valid but is not supported in this implementation.
    NotSupported = raw::TEE_ERROR_NOT_SUPPORTED,
    /// Expected data was missing.
    NoData = raw::TEE_ERROR_NO_DATA,
    /// System ran out of resources.
    OutOfMemory = raw::TEE_ERROR_OUT_OF_MEMORY,
    /// The system is busy working on something else.
    Busy = raw::TEE_ERROR_BUSY,
    /// Communication with a remote party failed.
    Communication = raw::TEE_ERROR_COMMUNICATION,
    /// A security fault was detected.
    Security = raw::TEE_ERROR_SECURITY,
    /// The supplied buffer is too short for the generated output.
    ShortBuffer = raw::TEE_ERROR_SHORT_BUFFER,
    /// The operation has been cancelled by an external event which occurred in
    /// the REE while the function was in progress.
    ExternalCancel = raw::TEE_ERROR_EXTERNAL_CANCEL,
    /// Data overflow.
    Overflow = raw::TEE_ERROR_OVERFLOW,
    /// Trusted Application has panicked during the operation.
    TargetDead = raw::TEE_ERROR_TARGET_DEAD,
    /// Insufficient space is available.
    StorageNoSpace = raw::TEE_ERROR_STORAGE_NO_SPACE,
    /// MAC is invalid.
    MacInvalid = raw::TEE_ERROR_MAC_INVALID,
    /// Signature is invalid.
    SignatureInvalid = raw::TEE_ERROR_SIGNATURE_INVALID,
    /// The persistent time has not been set.
    TimeNotSet = raw::TEE_ERROR_TIME_NOT_SET,
    /// The persistent time has been set but may have been corrupted and SHALL
    /// no longer be trusted.
    TimeNeedsReset = raw::TEE_ERROR_TIME_NEEDS_RESET,
    /// Unknown error, holding the original raw code.
    #[num_enum(catch_all)]
    Unknown(u32),
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::CorruptObject => "Object corruption.",
            ErrorKind::CorruptObject2 => "Persistent object corruption.",
            ErrorKind::StorageNotAvailable => "Object storage is not available.",
            ErrorKind::StorageNotAvailable2 => "Persistent object storage is not available.",
            ErrorKind::Generic => "Non-specific cause.",
            ErrorKind::AccessDenied => "Access privileges are not sufficient.",
            ErrorKind::Cancel => "The operation was canceled.",
            ErrorKind::AccessConflict => "Concurrent accesses caused conflict.",
            ErrorKind::ExcessData => "Too much data for the requested operation was passed.",
            ErrorKind::BadFormat => "Input data was of invalid format.",
            ErrorKind::BadParameters => "Input parameters were invalid.",
            ErrorKind::BadState => "Operation is not valid in the current state.",
            ErrorKind::ItemNotFound => "The requested data item is not found.",
            ErrorKind::NotImplemented => {
                "The requested operation should exist but is not yet implemented."
            }
            ErrorKind::NotSupported => {
                "The requested operation is valid but is not supported in this implementation."
            }
            ErrorKind::NoData => "Expected data was missing.",
            ErrorKind::OutOfMemory => "System ran out of resources.",
            ErrorKind::Busy => "The system is busy working on something else.",
            ErrorKind::Communication => "Communication with a remote party failed.",
            ErrorKind::Security => "A security fault was detected.",
            ErrorKind::ShortBuffer => "The supplied buffer is too short for the generated output.",
            ErrorKind::ExternalCancel => "Undocumented.",
            ErrorKind::Overflow => "Data overflow.",
            ErrorKind::TargetDead => "Trusted Application has panicked during the operation.",
            ErrorKind::StorageNoSpace => "Insufficient space is available.",
            ErrorKind::MacInvalid => "MAC is invalid.",
            ErrorKind::SignatureInvalid => "Signature is invalid.",
            ErrorKind::TimeNotSet => "The persistent time has not been set.",
            ErrorKind::TimeNeedsReset => {
                "The persistent time has been set but may have been corrupted and SHALL no longer be trusted."
            }
            ErrorKind::Unknown(_) => "Unknown error.",
        }
    }
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind, origin: None }
    }

    /// Creates a new instance of an `Error` from a particular TEE error code.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use optee_utee;
    ///
    /// let error = optee_utee::Error::from_raw_error(0xFFFF000F);
    /// assert_eq!(error.kind(), optee_utee::ErrorKind::Security);
    /// ```
    pub fn from_raw_error(code: u32) -> Error {
        Error {
            kind: ErrorKind::from(code),
            origin: None,
        }
    }

    pub fn with_origin(mut self, origin: ErrorOrigin) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Returns the corresponding `ErrorKind` for this error.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use optee_utee;
    ///
    /// let error = optee_utee::Error::new(optee_utee::ErrorKind::Security);
    /// ```
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the origin of this error.
    pub fn origin(&self) -> Option<ErrorOrigin> {
        self.origin.clone()
    }

    /// Returns raw code of this error.
    pub fn raw_code(&self) -> u32 {
        self.kind.into()
    }

    /// Returns corresponding error message of this error.
    pub fn message(&self) -> &str {
        self.kind().as_str()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{} (error code 0x{:x}, origin 0x{:x})",
            self.message(),
            self.raw_code(),
            self.origin().map(|v| v.into()).unwrap_or(0_u32),
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.message()
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error { kind, origin: None }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorOrigin {
    Api = raw::TEE_ORIGIN_API,
    Comms = raw::TEE_ORIGIN_COMMS,
    Tee = raw::TEE_ORIGIN_TEE,
    Ta = raw::TEE_ORIGIN_TRUSTED_APP,
    #[default]
    Unknown,
}

impl From<ErrorOrigin> for u32 {
    fn from(origin: ErrorOrigin) -> u32 {
        origin as u32
    }
}

impl From<u32> for ErrorOrigin {
    fn from(code: u32) -> ErrorOrigin {
        match code {
            raw::TEE_ORIGIN_API => ErrorOrigin::Api,
            raw::TEE_ORIGIN_COMMS => ErrorOrigin::Comms,
            raw::TEE_ORIGIN_TEE => ErrorOrigin::Tee,
            raw::TEE_ORIGIN_TRUSTED_APP => ErrorOrigin::Ta,
            _ => ErrorOrigin::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guarantees: unrecognized codes are preserved in the catch-all variant
    /// and round-trip through `Error`.
    #[test]
    fn unknown_raw_code_round_trips() {
        let code = 0x1234_5678;
        assert_eq!(ErrorKind::from(code), ErrorKind::Unknown(code));
        let back: u32 = ErrorKind::Unknown(code).into();
        assert_eq!(back, code);

        let err = Error::from_raw_error(code);
        assert_eq!(err.kind(), ErrorKind::Unknown(code));
        assert_eq!(err.raw_code(), code);
    }

    /// Guarantees: known codes still map to their named variants.
    #[test]
    fn known_codes_still_map() {
        assert_eq!(
            ErrorKind::from(raw::TEE_ERROR_SECURITY),
            ErrorKind::Security
        );
        let code: u32 = ErrorKind::Security.into();
        assert_eq!(code, raw::TEE_ERROR_SECURITY);
    }
}
