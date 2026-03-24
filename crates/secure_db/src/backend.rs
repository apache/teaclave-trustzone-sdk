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

use anyhow::{anyhow, bail, Result};
use optee_utee::{DataFlag, GenericObject, ObjectStorageConstants, PersistentObject};

// Wrapper functions for OP-TEE raw API

pub fn save_in_secure_storage(obj_id: &[u8], data: &[u8]) -> Result<()> {
    let obj_data_flag = DataFlag::ACCESS_READ
        | DataFlag::ACCESS_WRITE
        | DataFlag::ACCESS_WRITE_META
        | DataFlag::OVERWRITE;

    PersistentObject::create(
        ObjectStorageConstants::Private,
        obj_id,
        obj_data_flag,
        None,
        data,
    )
    .map_err(|e| anyhow!("[-] {:?}: failed to create object: {:?}", &obj_id, e))?;

    Ok(())
}

pub fn load_from_secure_storage(obj_id: &[u8]) -> Result<Option<Vec<u8>>> {
    match PersistentObject::open(
        ObjectStorageConstants::Private,
        obj_id,
        DataFlag::ACCESS_READ | DataFlag::SHARE_READ,
    ) {
        Err(e) => match e.kind() {
            optee_utee::ErrorKind::ItemNotFound => Ok(None),
            _ => {
                bail!("[-] {:?}: failed to open object: {:?}", &obj_id, e);
            }
        },

        Ok(mut object) => {
            let obj_info = object.info()?;
            let mut buf = vec![0u8; obj_info.data_size()];

            let read_bytes = object.read(&mut buf)?;
            if read_bytes != obj_info.data_size() as u32 {
                bail!("[-] {:?}: failed to read data", &obj_id);
            }

            Ok(Some(buf))
        }
    }
}

pub fn delete_from_secure_storage(obj_id: &[u8]) -> Result<()> {
    match PersistentObject::open(
        ObjectStorageConstants::Private,
        obj_id,
        DataFlag::ACCESS_READ | DataFlag::ACCESS_WRITE_META,
    ) {
        Err(e) => {
            bail!("[-] {:?}: failed to open object: {:?}", &obj_id, e);
        }

        Ok(object) => {
            object.close_and_delete()?;
            Ok(())
        }
    }
}
