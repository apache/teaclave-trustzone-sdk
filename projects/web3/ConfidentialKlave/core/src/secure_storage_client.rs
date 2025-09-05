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

use anyhow::Result;
use secure_storage::SecureStorageDb;
use std::{
    collections::HashMap,
    convert::TryFrom,
    hash::Hash,
    sync::{Arc, RwLock},
};
use types::Storable;

pub struct SecureStorageClient {
    db: Arc<RwLock<SecureStorageDb>>,
}

impl SecureStorageClient {
    pub fn init() -> Self {
        Self {
            db: Arc::new(RwLock::new(
                SecureStorageDb::open("db".to_string()).unwrap(),
            )),
        }
    }

    pub fn get<K, V>(&self, key: &K) -> Result<V>
    where
        K: TryFrom<String> + Into<String> + Clone,
        V: Storable<K>,
    {
        let key: String = (*key).clone().into();
        let storage_key = V::concat_key(&key);
        let value = self.db.read().unwrap().get(&storage_key)?;
        // if entry is not found, return Err
        Ok(bincode::deserialize(&value)?)
    }

    pub fn put<K, V>(&self, value: &V) -> Result<()>
    where
        K: TryFrom<String> + Into<String> + Clone,
        V: Storable<K>,
    {
        let key = value.storage_key();
        let value = bincode::serialize(value)?;
        self.db.write().unwrap().put(key, value)?;
        Ok(())
    }

    pub fn delete_entry<K, V>(&self, key: &K) -> Result<()>
    where
        K: TryFrom<String> + Into<String> + Clone,
        V: Storable<K>,
    {
        let key: String = (*key).clone().into();
        let storage_key = V::concat_key(&key);
        self.db.write().unwrap().delete(storage_key)?;
        Ok(())
    }

    pub fn list_entries<K, V>(&self) -> Result<HashMap<K, V>>
    where
        K: TryFrom<String> + Into<String> + Clone + Eq + Hash,
        V: Storable<K>,
    {
        let map = self
            .db
            .read()
            .unwrap()
            .list_entries_with_prefix(V::table_name())?;
        let mut result = HashMap::new();
        for (_k, v) in map {
            let value: V = bincode::deserialize(&v)?;
            let key = value.unique_id();
            result.insert(key, value);
        }
        Ok(result)
    }
}
