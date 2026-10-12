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

use crate::Uuid;
use num_enum::{FromPrimitive, IntoPrimitive};
use optee_utee_sys as raw;

#[derive(Copy, Clone)]
pub struct Identity {
    raw: raw::TEE_Identity,
}

impl Identity {
    /// Returns the login type of this client identity.
    ///
    /// The GlobalPlatform specification reserves implementation-defined login ranges,
    /// so a login value outside the set named below is possible in principle. Such a
    /// value is returned as [`LoginType::Unknown`] carrying the raw code instead of
    /// aborting the TA, so a TA inspecting its callers must decide explicitly what to
    /// do with a login it does not recognize — normally by rejecting the caller.
    pub fn login_type(&self) -> LoginType {
        LoginType::from(self.raw.login)
    }

    pub fn uuid(&self) -> Uuid {
        Uuid::from(self.raw.uuid)
    }
}

impl From<raw::TEE_Identity> for Identity {
    fn from(raw: raw::TEE_Identity) -> Self {
        Self { raw }
    }
}

/// The login type of a client, as stamped by OP-TEE into `TEE_Identity::login`.
///
/// This is the Rust-side mirror of the `TEE_LOGIN_*` constants defined in the C
/// header. The `Unknown(u32)` variant catches any login value this crate does not
/// name, so the value always round-trips back through [`IntoPrimitive`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive, strum::Display)]
#[repr(u32)]
pub enum LoginType {
    Public = raw::TEE_LOGIN_PUBLIC,
    User = raw::TEE_LOGIN_USER,
    Group = raw::TEE_LOGIN_GROUP,
    Application = raw::TEE_LOGIN_APPLICATION,
    ApplicationUser = raw::TEE_LOGIN_APPLICATION_USER,
    ApplicationGroup = raw::TEE_LOGIN_APPLICATION_GROUP,
    TrustedApp = raw::TEE_LOGIN_TRUSTED_APP,
    #[num_enum(catch_all)]
    Unknown(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_with_login(login: u32) -> Identity {
        Identity::from(raw::TEE_Identity {
            login,
            uuid: raw::TEE_UUID {
                timeLow: 0,
                timeMid: 0,
                timeHiAndVersion: 0,
                clockSeqAndNode: [0; 8],
            },
        })
    }

    /// Guarantees: the login values named by the GlobalPlatform specification still
    /// map to their named variants.
    #[test]
    fn known_login_types_still_map() {
        let known = [
            (raw::TEE_LOGIN_PUBLIC, LoginType::Public),
            (raw::TEE_LOGIN_USER, LoginType::User),
            (raw::TEE_LOGIN_GROUP, LoginType::Group),
            (raw::TEE_LOGIN_APPLICATION, LoginType::Application),
            (raw::TEE_LOGIN_APPLICATION_USER, LoginType::ApplicationUser),
            (
                raw::TEE_LOGIN_APPLICATION_GROUP,
                LoginType::ApplicationGroup,
            ),
            (raw::TEE_LOGIN_TRUSTED_APP, LoginType::TrustedApp),
        ];
        for (raw_login, expected) in known {
            assert_eq!(identity_with_login(raw_login).login_type(), expected);
        }
    }

    /// Guarantees: reading a login value outside the named set yields `Unknown`
    /// instead of panicking, and the raw value round-trips.
    #[test]
    fn unnamed_login_types_do_not_panic() {
        for raw_login in [0x0000_0003, 0x0000_0300, 0x8000_0000, u32::MAX] {
            let login_type = identity_with_login(raw_login).login_type();
            assert_eq!(login_type, LoginType::Unknown(raw_login));
            assert_eq!(u32::from(login_type), raw_login);
        }
    }
}
