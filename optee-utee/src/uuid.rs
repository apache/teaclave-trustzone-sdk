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

use core::fmt;
use hex;
use optee_utee_sys as raw;
use uuid as uuid_crate;
use crate::{ErrorKind, Result};

/// A Universally Unique Resource Identifier (UUID) type as defined in RFC4122.
/// The value is used to identify a trusted application.
#[derive(Copy, Clone)]
pub struct Uuid {
    raw: raw::TEE_UUID,
}

impl Uuid {
    /// Parses a Uuid from a string of hexadecimal digits with optional hyphens.
    ///
    /// # Examples
    ///
    /// ``` rust,no_run
    /// # use optee_utee::Uuid;
    /// # fn main() -> optee_utee::Result<()> {
    ///
    /// let uuid = Uuid::parse_str("8abcf200-2450-11e4-abe2-0002a5d5c51b")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_str(input: &str) -> Result<Uuid> {
        let uuid = uuid_crate::Uuid::parse_str(input).map_err(|_| ErrorKind::BadFormat)?;
        let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = uuid.as_fields();
        Ok(Self::new_raw(
            time_low,
            time_mid,
            time_hi_and_version,
            *clock_seq_and_node,
        ))
    }

    /// Creates a `Uuid` using the supplied big-endian bytes.
    ///
    /// # Examples
    ///
    /// ``` rust,no_run
    /// # use optee_utee::Uuid;
    /// let bytes: [u8; 16] = [70, 235, 208, 238, 14, 109, 67, 201, 185, 13, 204, 195, 90, 145, 63, 62,];
    /// let uuid = Uuid::from_bytes(bytes);
    /// ```
    pub fn from_bytes(bytes: [u8; 16]) -> Uuid {
        let uuid = uuid_crate::Uuid::from_bytes(bytes);
        let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = uuid.as_fields();
        Self::new_raw(time_low, time_mid, time_hi_and_version, *clock_seq_and_node)
    }

    /// Creates a `Uuid` using a slice of supplied big-endian bytes.
    ///
    /// # Examples
    ///
    /// ``` rust,no_run
    /// # use optee_utee::Uuid;
    /// # fn main() -> optee_utee::Result<()> {
    /// let bytes: &[u8; 16] = &[70, 235, 208, 238, 14, 109, 67, 201, 185, 13, 204, 195, 90, 145, 63, 62,];
    /// let uuid = Uuid::from_slice(bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_slice(b: &[u8]) -> Result<Uuid> {
        let uuid = uuid_crate::Uuid::from_slice(b).map_err(|_| ErrorKind::BadFormat)?;
        let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = uuid.as_fields();
        Ok(Self::new_raw(
            time_low,
            time_mid,
            time_hi_and_version,
            *clock_seq_and_node,
        ))
    }

    /// Creates a raw TEE client uuid object with specified parameters.
    pub fn new_raw(
        time_low: u32,
        time_mid: u16,
        time_hi_and_version: u16,
        clock_seq_and_nod: [u8; 8],
    ) -> Uuid {
        let raw_uuid = raw::TEE_UUID {
            timeLow: time_low,
            timeMid: time_mid,
            timeHiAndVersion: time_hi_and_version,
            clockSeqAndNode: clock_seq_and_nod,
        };
        Self { raw: raw_uuid }
    }

    /// Converts a uuid to a const raw `TEE_UUID` pointer.
    pub fn as_raw_ptr(&self) -> *const raw::TEE_UUID {
        &self.raw
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{}-{}",
            self.raw.timeLow,
            self.raw.timeMid,
            self.raw.timeHiAndVersion,
            hex::encode(&self.raw.clockSeqAndNode[0..2]),
            hex::encode(&self.raw.clockSeqAndNode[2..8]),
        )
    }
}

impl From<raw::TEE_UUID> for Uuid {
    fn from(raw: raw::TEE_UUID) -> Self {
        Uuid { raw }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_to_string() {
        let uuids = [
            "00173366-2aca-49bc-beb7-10c975e6131e", // uuid with timeLow leading zeros
            "11173366-0aca-49bc-beb7-10c975e6131e", // uuid with timeMid leading zeros
            "11173366-2aca-09bc-beb7-10c975e6131e", // uuid with timeHiAndVersion leading zeros
            "11173366-2aca-19bc-beb7-10c975e6131e", // random uuid
        ];
        for origin in uuids.iter() {
            let uuid = Uuid::parse_str(origin).expect("Test UUID should be valid");
            let formatted = uuid.to_string();
            assert_eq!(*origin, formatted);
        }
    }
}
