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

use crate::raw;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::fmt;

/// A specialized [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for TEE operations.
///
/// # Examples
///
/// ``` no_run
/// use optee_teec::Context;
///
/// fn main() -> optee_teec::Result<()> {
///     let mut ctx = Context::new()?;
///     Ok(())
/// }
/// ````
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for TEE operations of [`Context`] and [`Session`].
///
/// [`Context`]: struct.Context.html
/// [`Session`]: struct.Session.html
#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
    origin: Option<ErrorOrigin>,
}

/// A list specifying general categories of TEE client error and its
/// corresponding code in OP-TEE client library.
#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, FromPrimitive, IntoPrimitive,
)]
#[repr(u32)]
pub enum ErrorKind {
    /// Non-specific cause.
    Generic = raw::TEEC_ERROR_GENERIC,
    /// Access privileges are not sufficient.
    AccessDenied = raw::TEEC_ERROR_ACCESS_DENIED,
    /// The operation was canceled.
    Cancel = raw::TEEC_ERROR_CANCEL,
    /// Concurrent accesses caused conflict.
    AccessConflict = raw::TEEC_ERROR_ACCESS_CONFLICT,
    /// Too much data for the requested operation was passed.
    ExcessData = raw::TEEC_ERROR_EXCESS_DATA,
    /// Input data was of invalid format.
    BadFormat = raw::TEEC_ERROR_BAD_FORMAT,
    /// Input parameters were invalid.
    BadParameters = raw::TEEC_ERROR_BAD_PARAMETERS,
    /// Operation is not valid in the current state.
    BadState = raw::TEEC_ERROR_BAD_STATE,
    /// The requested data item is not found.
    ItemNotFound = raw::TEEC_ERROR_ITEM_NOT_FOUND,
    /// The requested operation should exist but is not yet implemented.
    NotImplemented = raw::TEEC_ERROR_NOT_IMPLEMENTED,
    /// The requested operation is valid but is not supported in this implementation.
    NotSupported = raw::TEEC_ERROR_NOT_SUPPORTED,
    /// Expected data was missing.
    NoData = raw::TEEC_ERROR_NO_DATA,
    /// System ran out of resources.
    OutOfMemory = raw::TEEC_ERROR_OUT_OF_MEMORY,
    /// The system is busy working on something else.
    Busy = raw::TEEC_ERROR_BUSY,
    /// Communication with a remote party failed.
    Communication = raw::TEEC_ERROR_COMMUNICATION,
    /// A security fault was detected.
    Security = raw::TEEC_ERROR_SECURITY,
    /// The supplied buffer is too short for the generated output.
    ShortBuffer = raw::TEEC_ERROR_SHORT_BUFFER,
    /// Implementation defined error code.
    ExternalCancel = raw::TEEC_ERROR_EXTERNAL_CANCEL,
    /// Implementation defined error code: trusted Application has panicked during the operation.
    TargetDead = raw::TEEC_ERROR_TARGET_DEAD,
    /// Unknown error.
    #[default]
    Unknown,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
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
            ErrorKind::TargetDead => "Trusted Application has panicked during the operation.",
            ErrorKind::Unknown => "Unknown error.",
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
    /// ```
    /// use optee_teec::{Error, ErrorKind};
    ///
    /// let error = Error::from_raw_error(0xFFFF000F);
    /// assert_eq!(error.kind(), ErrorKind::Security);
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
    /// ```
    /// use optee_teec::{Error, ErrorKind};
    ///
    /// let error = Error::new(ErrorKind::Security);
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

impl std::error::Error for Error {
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

#[derive(Clone, Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ErrorOrigin {
    API = raw::TEEC_ORIGIN_API,
    COMMS = raw::TEEC_ORIGIN_COMMS,
    TEE = raw::TEEC_ORIGIN_TEE,
    TA = raw::TEEC_ORIGIN_TRUSTED_APP,
    #[default]
    UNKNOWN,
}
