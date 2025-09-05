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
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use crate::user::{ADMIN_ID, SYSTEM_ID};
use crate::{ROOT_PUBKEY, SYSTEM_PUBKEY};
use basic_utils::println;
use types::share::{CkHash, CkHasher, CkPublicKey, Role, RoleSet, TaUserInfo, UserID};

pub struct UserRegistry {
    pub inner: HashMap<CkHash, TaUserInfo>,
}

impl UserRegistry {
    pub fn init() -> Self {
        let mut inner = HashMap::new();

        // hardcoded admin
        let admin_pubkey_hash = match CkPublicKey::new(ROOT_PUBKEY.to_vec()).hash() {
            Ok(hash) => hash,
            Err(_) => {
                println!("[-] UserRegistry::init(): failed to hash admin pubkey, set as default");
                CkHash::default()
            }
        };
        let admin_info = TaUserInfo::new(
            admin_pubkey_hash.clone(),
            UserID::new(ADMIN_ID),
            RoleSet(vec![Role::Admin]),
        );
        inner.insert(admin_pubkey_hash, admin_info);

        // hardcoded system
        let system_pubkey_hash = match CkPublicKey::new(SYSTEM_PUBKEY.to_vec()).hash() {
            Ok(hash) => hash,
            Err(_) => {
                println!("[-] UserRegistry::init(): failed to hash system pubkey, set as default");
                CkHash::default()
            }
        };
        let system_info = TaUserInfo::new(
            system_pubkey_hash.clone(),
            UserID::new(SYSTEM_ID),
            RoleSet(vec![Role::System]),
        );
        inner.insert(system_pubkey_hash, system_info);

        Self { inner }
    }

    pub fn auth_as_role<U: TryFrom<TaUserInfo>>(&self, pubkey_hash: &CkHash) -> Result<U> {
        let user_info = self
            .inner
            .get(pubkey_hash)
            .ok_or_else(|| anyhow!("[-] UserRegistry::auth_as_role(): pubkey not found"))?;
        U::try_from(user_info.clone())
            .map_err(|_e| anyhow!("[-] UserRegistry::auth_as_role() convert error"))
    }

    pub fn set_users(&mut self, user_info: Vec<TaUserInfo>) {
        for info in user_info {
            self.inner.insert(info.pubkey_hash().clone(), info);
        }
    }

    pub fn get_accepted_pubkey_hash(&self) -> HashSet<CkHash> {
        let mut pubkey_hash_set = HashSet::new();
        for pubkey_hash in self.inner.keys() {
            pubkey_hash_set.insert(pubkey_hash.clone());
        }
        pubkey_hash_set
    }
}
