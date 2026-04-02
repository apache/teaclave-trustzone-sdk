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
use optee_utee_sys as raw;
use strum_macros::Display;

#[derive(Copy, Clone)]
pub struct Identity {
    raw: raw::TEE_Identity,
}

impl Identity {
    pub fn login_type(&self) -> LoginType {
        match self.raw.login {
            raw::TEE_LOGIN_PUBLIC => LoginType::Public,
            raw::TEE_LOGIN_USER => LoginType::User,
            raw::TEE_LOGIN_GROUP => LoginType::Group,
            raw::TEE_LOGIN_APPLICATION => LoginType::Application,
            raw::TEE_LOGIN_APPLICATION_USER => LoginType::ApplicationUser,
            raw::TEE_LOGIN_APPLICATION_GROUP => LoginType::ApplicationGroup,
            raw::TEE_LOGIN_TRUSTED_APP => LoginType::TrustedApp,
            _ => panic!("Invalid login type"),
        }
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

#[derive(Copy, Clone, PartialEq, Eq, Display)]
#[repr(u32)]
pub enum LoginType {
    Public = raw::TEE_LOGIN_PUBLIC,
    User = raw::TEE_LOGIN_USER,
    Group = raw::TEE_LOGIN_GROUP,
    Application = raw::TEE_LOGIN_APPLICATION,
    ApplicationUser = raw::TEE_LOGIN_APPLICATION_USER,
    ApplicationGroup = raw::TEE_LOGIN_APPLICATION_GROUP,
    TrustedApp = raw::TEE_LOGIN_TRUSTED_APP,
}
