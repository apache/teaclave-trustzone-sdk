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
use std::io::{Read, Write};

pub fn save_in_secure_storage(obj_id: &[u8], data: &[u8]) -> Result<()> {
    // save data to file, file name is hex(obj_id)
    let file_name = hex::encode(obj_id);
    let mut file = std::fs::File::create(&file_name)?;
    file.write_all(data).map_err(|e| {
        anyhow!(
            "Failed to write data to file: {}, error: {}",
            &file_name,
            e.to_string()
        )
    })
}

pub fn load_from_secure_storage(obj_id: &[u8]) -> Result<Vec<u8>> {
    // load data from file, file name is hex(obj_id)
    let file_name = hex::encode(obj_id);
    let mut file = std::fs::File::open(file_name)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

pub fn delete_from_secure_storage(obj_id: Vec<u8>) -> Result<()> {
    // delete file, file name is hex(obj_id)
    let file_name = hex::encode(obj_id);
    std::fs::remove_file(file_name)?;
    Ok(())
}
