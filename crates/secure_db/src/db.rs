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

use crate::{delete_from_secure_storage, load_from_secure_storage, save_in_secure_storage};
use anyhow::{anyhow, ensure, Result};
use hashbrown::HashSet;
use std::collections::HashMap;

// SecureStorageDb is a key-value storage for TA to easily store and retrieve data.
// First we store the key list in the secure storage, named as db_name.
// Then we store the each key-value pairs in the secure storage.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureStorageDb {
    name: String,
    key_list: HashSet<String>,
}

impl SecureStorageDb {
    pub fn open(name: String) -> Result<Self> {
        let key_list = match load_from_secure_storage(name.as_bytes())? {
            Some(data) => bincode::deserialize(&data)?,
            None => {
                // create new db
                //
                // Note: `std::collections::HashSet` was replaced with
                // `hashbrown::HashSet`, due to a write permission fault
                // observed during testing. The exact cause of the issue is
                // unclear, but using `hashbrown::HashSet` resolves it.
                HashSet::new()
            }
        };
        Ok(Self { name, key_list })
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        save_in_secure_storage(key.as_bytes(), &value)
            .map_err(|e| anyhow!("[+] SecureStorage::insert(): save error: {}", e))?;
        self.key_list.insert(key);
        self.store_key_list()
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        ensure!(self.key_list.contains(key), "Key not found in key list");
        load_from_secure_storage(key.as_bytes())
            .map_err(|e| anyhow!("[+] SecureStorage::get(): load error: {}", e))?
            .ok_or_else(|| anyhow!("[+] SecureStorage::get(): object not found in db"))
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        // ensure key must exist
        ensure!(self.key_list.contains(key), "Key not found in key list");
        delete_from_secure_storage(key.as_bytes())
            .map_err(|e| anyhow!("[+] SecureStorage::delete(): delete error: {}", e))?;
        self.key_list.remove(key);
        self.store_key_list()
    }

    pub fn clear(&mut self) -> Result<()> {
        for key in self.key_list.clone() {
            self.delete(&key)?;
        }
        Ok(())
    }

    pub fn list_entries_with_prefix(&self, prefix: &str) -> Result<HashMap<String, Vec<u8>>> {
        let mut result = HashMap::new();
        for key in &self.key_list {
            if key.starts_with(prefix) {
                let value = self.get(key)?;
                result.insert(key.clone(), value);
            }
        }
        Ok(result)
    }

    fn store_key_list(&self) -> Result<()> {
        let key_list = bincode::serialize(&self.key_list)?;
        save_in_secure_storage(self.name.as_bytes(), &key_list)?;
        Ok(())
    }
}
