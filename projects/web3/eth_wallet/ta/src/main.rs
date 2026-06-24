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

mod hash;
mod wallet;

use optee_utee::prelude::*;
use optee_utee::{Error, ErrorKind};
use proto::Command;
use secure_db::SecureStorageClient;

use anyhow::{anyhow, bail, Result};
use wallet::Wallet;

const DB_NAME: &str = "eth_wallet_db";

#[ta_create]
fn create() -> optee_utee::Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut ParametersNone) -> optee_utee::Result<()> {
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

#[cfg(debug_assertions)]
macro_rules! dbg_println {
    ($($arg:tt)*) => (trace_println!($($arg)*));
}

#[cfg(not(debug_assertions))]
macro_rules! dbg_println {
    ($($arg:tt)*) => {};
}

fn create_wallet(_input: &proto::CreateWalletInput) -> Result<proto::CreateWalletOutput> {
    let wallet = Wallet::new()?;
    let wallet_id = wallet.get_id();
    let mnemonic = wallet.get_mnemonic()?;
    dbg_println!("[+] Wallet ID: {:?}", wallet_id);

    let db_client = SecureStorageClient::open(DB_NAME)?;
    db_client.put(&wallet)?;
    dbg_println!("[+] Wallet saved in secure storage");

    Ok(proto::CreateWalletOutput {
        wallet_id,
        mnemonic,
    })
}

fn remove_wallet(input: &proto::RemoveWalletInput) -> Result<proto::RemoveWalletOutput> {
    dbg_println!("[+] Removing wallet: {:?}", input.wallet_id);

    let db_client = SecureStorageClient::open(DB_NAME)?;
    db_client.delete_entry::<Wallet>(&input.wallet_id)?;
    dbg_println!("[+] Wallet removed");

    Ok(proto::RemoveWalletOutput {})
}

fn derive_address(input: &proto::DeriveAddressInput) -> Result<proto::DeriveAddressOutput> {
    let db_client = SecureStorageClient::open(DB_NAME)?;
    let wallet = db_client
        .get::<Wallet>(&input.wallet_id)
        .map_err(|e| anyhow!("[+] Deriving address: error: wallet not found: {:?}", e))?;
    dbg_println!("[+] Deriving address: wallet loaded");

    let (address, public_key) = wallet.derive_address(&input.hd_path)?;
    dbg_println!("[+] Deriving address: address: {:?}", address);
    dbg_println!("[+] Deriving address: public key: {:?}", public_key);

    Ok(proto::DeriveAddressOutput {
        address,
        public_key,
    })
}

fn sign_transaction(input: &proto::SignTransactionInput) -> Result<proto::SignTransactionOutput> {
    let db_client = SecureStorageClient::open(DB_NAME)?;
    let wallet = db_client
        .get::<Wallet>(&input.wallet_id)
        .map_err(|e| anyhow!("[+] Sign transaction: error: wallet not found: {:?}", e))?;
    dbg_println!("[+] Sign transaction: wallet loaded");

    let signature = wallet.sign_transaction(&input.hd_path, &input.transaction)?;
    dbg_println!("[+] Sign transaction: signature: {:?}", signature);

    Ok(proto::SignTransactionOutput { signature })
}

fn handle_invoke(command: Command, serialized_input: &[u8]) -> Result<Vec<u8>> {
    fn process<T: serde::de::DeserializeOwned, U: serde::Serialize, F: Fn(&T) -> Result<U>>(
        serialized_input: &[u8],
        handler: F,
    ) -> Result<Vec<u8>> {
        let input: T = bincode::deserialize(serialized_input)?;
        let output = handler(&input)?;
        let serialized_output = bincode::serialize(&output)?;
        Ok(serialized_output)
    }

    match command {
        Command::CreateWallet => process(serialized_input, create_wallet),
        Command::RemoveWallet => process(serialized_input, remove_wallet),
        Command::DeriveAddress => process(serialized_input, derive_address),
        Command::SignTransaction => process(serialized_input, sign_transaction),
        _ => bail!("Unsupported command"),
    }
}

#[ta_invoke_command]
fn invoke_command(
    cmd_id: u32,
    (p0, p1, _, _): &mut (
        ParameterMemrefInput<'_>,
        ParameterMemrefOutput<'_>,
        ParameterNone,
        ParameterNone,
    ),
) -> optee_utee::Result<()> {
    dbg_println!("[+] TA invoke command");

    p1.set_updated_size(0)?;
    let output_vec = match handle_invoke(Command::from(cmd_id), p0.get_buffer()) {
        Ok(output) => output,
        Err(e) => {
            let err_message = format!("{:?}", e);
            p1.set_output(err_message)?;
            return Err(Error::new(ErrorKind::BadParameters));
        }
    };
    p1.set_output(output_vec)?;
    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
