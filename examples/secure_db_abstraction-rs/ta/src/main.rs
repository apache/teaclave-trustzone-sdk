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

#![no_main]

extern crate alloc;

use alloc::vec;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::Command;
use secure_db::{SecureStorageClient, Storable};
use serde::{Deserialize, Serialize};

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Test => match test() {
            Ok(_) => {
                trace_println!("[+] Test passed");
                Ok(())
            }
            Err(e) => {
                trace_println!("[-] Test failed: {:?}", e);
                Err(Error::new(ErrorKind::Generic))
            }
        },
        _ => {
            return Err(Error::new(ErrorKind::NotSupported));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ExampleData {
    pub id: String,
    pub data: Vec<u8>,
}

// Any structure that implements Storable can be stored in the secure db.
// Any Key type can be used as unique id as long as it implements
// TryFrom<String> + ToString
impl Storable for ExampleData {
    type Key = String;

    fn unique_id(&self) -> Self::Key {
        self.id.clone()
    }
}

pub fn test() -> anyhow::Result<()> {
    // Assume this is the data that we want to store
    let example_data = ExampleData {
        id: "example_data".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };

    // Initialize secure storage db client with a db name
    let db_client = SecureStorageClient::open("secure_db")?;

    // Now, we can do common db interactions using the db_client:
    // Store data in db using put()
    db_client.put(&example_data)?;
    // Load data from db using get()
    let loaded_example_data = db_client.get::<ExampleData>(&example_data.id)?;
    anyhow::ensure!(
        loaded_example_data == example_data,
        "Loaded example_data is not equal to the original example_data"
    );
    // List all entries in db
    let entries = db_client.list_entries::<ExampleData>()?;
    trace_println!("Entries: {:?}", entries);
    // Delete entry from db
    db_client.delete_entry::<ExampleData>(&example_data.id)?;

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
