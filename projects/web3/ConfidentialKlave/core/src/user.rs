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

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use types::share::TaUserInfo;

pub const ADMIN_ID: [u8; 8] = [0x00; 8];
pub const SYSTEM_ID: [u8; 8] = [0x01; 8];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Admin(pub TaUserInfo);

impl TryFrom<TaUserInfo> for Admin {
    type Error = anyhow::Error;

    fn try_from(info: TaUserInfo) -> Result<Self> {
        if info.is_admin() {
            Ok(Self(info))
        } else {
            Err(anyhow!("[-] Admin::try_from(): not an admin"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct System(pub TaUserInfo);

impl TryFrom<TaUserInfo> for System {
    type Error = anyhow::Error;

    fn try_from(info: TaUserInfo) -> Result<Self> {
        if info.is_system() {
            Ok(Self(info))
        } else {
            Err(anyhow!("[-] System::try_from(): not a system"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Approver(pub TaUserInfo);

impl TryFrom<TaUserInfo> for Approver {
    type Error = anyhow::Error;

    fn try_from(info: TaUserInfo) -> Result<Self> {
        if info.is_approver() {
            Ok(Self(info))
        } else {
            Err(anyhow!("[-] Approver::try_from(): not an approver"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TxOperator(pub TaUserInfo);

impl TryFrom<TaUserInfo> for TxOperator {
    type Error = anyhow::Error;

    fn try_from(info: TaUserInfo) -> Result<Self> {
        if info.is_tx_operator() {
            Ok(Self(info))
        } else {
            Err(anyhow!("[-] TxOperator::try_from(): not a tx operator"))
        }
    }
}

impl TryFrom<TaUserInfo> for TaUser {
    type Error = anyhow::Error;

    fn try_from(info: TaUserInfo) -> Result<Self> {
        if info.is_admin() {
            Ok(TaUser::Admin(Admin(info)))
        } else if info.is_system() {
            Ok(TaUser::System(System(info)))
        } else if info.is_approver() {
            Ok(TaUser::Approver(Approver(info)))
        } else if info.is_tx_operator() {
            Ok(TaUser::TxOperator(TxOperator(info)))
        } else {
            Ok(TaUser::Unauthorized)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaUser {
    Admin(Admin),
    System(System),
    Approver(Approver),
    TxOperator(TxOperator),
    Unauthorized,
}

impl TaUser {
    pub fn is_admin(&self) -> bool {
        match self {
            TaUser::Admin(_) => true,
            _ => false,
        }
    }

    pub fn is_approver(&self) -> bool {
        match self {
            TaUser::Approver(_) => true,
            _ => false,
        }
    }

    pub fn is_tx_operator(&self) -> bool {
        match self {
            TaUser::TxOperator(_) => true,
            _ => false,
        }
    }

    pub fn is_system(&self) -> bool {
        match self {
            TaUser::System(_) => true,
            _ => false,
        }
    }
}

impl From<Admin> for TaUser {
    fn from(admin: Admin) -> Self {
        TaUser::Admin(admin)
    }
}

impl From<Approver> for TaUser {
    fn from(approver: Approver) -> Self {
        TaUser::Approver(approver)
    }
}

impl From<TxOperator> for TaUser {
    fn from(tx_operator: TxOperator) -> Self {
        TaUser::TxOperator(tx_operator)
    }
}

impl From<System> for TaUser {
    fn from(system: System) -> Self {
        TaUser::System(system)
    }
}
