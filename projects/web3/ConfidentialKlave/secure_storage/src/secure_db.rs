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
use anyhow::{bail, ensure, Result};
use basic_utils::println;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureStorageDb {
    name: String,
    key_list: HashSet<String>,
}

impl SecureStorageDb {
    pub fn open(name: String) -> Result<Self> {
        match load_from_secure_storage(name.as_bytes()) {
            Ok(data) => {
                let key_list = bincode::deserialize(&data)?;
                Ok(Self { name, key_list })
            }
            Err(e) => {
                println!(
                    "[-] SecureStorage::load(): load error: {}. Create new...",
                    e
                );
                Ok(Self {
                    name,
                    key_list: HashSet::new(),
                })
            }
        }
    }

    pub fn put(&mut self, key: String, value: Vec<u8>) -> Result<()> {
        match save_in_secure_storage(key.as_bytes(), &value) {
            Ok(_) => {
                self.key_list.insert(key);
                self.store_key_list()?;
            }
            Err(e) => {
                bail!("[+] SecureStorage::insert(): save error: {}", e);
            }
        };
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        ensure!(self.key_list.contains(key), "Key not found");
        match load_from_secure_storage(key.as_bytes()) {
            Ok(data) => Ok(data),
            Err(e) => {
                bail!("[+] SecureStorage::get(): load error: {}", e);
            }
        }
    }

    pub fn delete(&mut self, key: String) -> Result<()> {
        match delete_from_secure_storage(key.as_bytes().to_vec()) {
            Ok(_) => {
                self.key_list.remove(&key);
                self.store_key_list()?;
            }
            Err(e) => {
                bail!("[+] SecureStorage::delete(): delete error: {}", e);
            }
        };
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for key in self.key_list.clone() {
            self.delete(key)?;
        }
        self.key_list.clear();
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
