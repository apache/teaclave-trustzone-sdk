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

mod cli;
mod tests;

use optee_teec::{Context, Operation, Uuid};
use optee_teec::{ParamNone, ParamTmpRef};

use anyhow::Result;
use structopt::StructOpt;

const OUTPUT_MAX_SIZE: usize = 1024;

fn invoke_command(command: proto::Command, input: &[u8]) -> optee_teec::Result<Vec<u8>> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(proto::UUID)
        .map_err(|_| optee_teec::Error::new(optee_teec::ErrorKind::ItemNotFound))?;
    let mut session = ctx.open_session(uuid)?;

    println!("CA: command: {:?}", command);
    // input buffer
    let p0 = ParamTmpRef::new_input(input);
    // output buffer
    let mut output = vec![0u8; OUTPUT_MAX_SIZE];
    let p1 = ParamTmpRef::new_output(output.as_mut_slice());

    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);
    match session.invoke_command(command as u32, &mut operation) {
        Ok(()) => {
            println!("CA: invoke_command success");
            let output_len = operation.parameters().1.updated_size();
            Ok(output[..output_len].to_vec())
        }
        Err(e) => {
            let output_len = operation.parameters().1.updated_size();
            let err_message = String::from_utf8_lossy(&output[..output_len]);
            println!("CA: invoke_command failed: {:?}", err_message);
            Err(e)
        }
    }
}

pub fn create_wallet() -> Result<uuid::Uuid> {
    let serialized_output = invoke_command(proto::Command::CreateWallet, &[])?;
    let output: proto::CreateWalletOutput = bincode::deserialize(&serialized_output)?;
    Ok(output.wallet_id)
}

pub fn remove_wallet(wallet_id: uuid::Uuid) -> Result<()> {
    let input = proto::RemoveWalletInput { wallet_id };
    let _output = invoke_command(proto::Command::RemoveWallet, &bincode::serialize(&input)?)?;
    Ok(())
}

pub fn derive_address(wallet_id: uuid::Uuid, hd_path: &str) -> Result<[u8; 20]> {
    let input = proto::DeriveAddressInput {
        wallet_id,
        hd_path: hd_path.to_string(),
    };
    let serialized_output =
        invoke_command(proto::Command::DeriveAddress, &bincode::serialize(&input)?)?;
    let output: proto::DeriveAddressOutput = bincode::deserialize(&serialized_output)?;
    Ok(output.address)
}

pub fn sign_transaction(
    wallet_id: uuid::Uuid,
    hd_path: &str,
    chain_id: u64,
    nonce: u128,
    to: [u8; 20],
    value: u128,
    gas_price: u128,
    gas: u128,
) -> Result<Vec<u8>> {
    let transaction = proto::EthTransaction {
        chain_id,
        nonce,
        to: Some(to),
        value,
        gas_price,
        gas,
        data: vec![],
    };
    let input = proto::SignTransactionInput {
        wallet_id,
        hd_path: hd_path.to_string(),
        transaction,
    };
    let serialized_output = invoke_command(
        proto::Command::SignTransaction,
        &bincode::serialize(&input)?,
    )?;
    let output: proto::SignTransactionOutput = bincode::deserialize(&serialized_output)?;
    Ok(output.signature)
}

fn main() -> Result<()> {
    let args = cli::Opt::from_args();
    match args.command {
        cli::Command::CreateWallet(_opt) => {
            let wallet_id = create_wallet()?;
            println!("Wallet ID: {}", wallet_id);
        }
        cli::Command::RemoveWallet(opt) => {
            remove_wallet(opt.wallet_id)?;
            println!("Wallet removed");
        }
        cli::Command::DeriveAddress(opt) => {
            let address = derive_address(opt.wallet_id, &opt.hd_path)?;
            println!("Address: 0x{}", hex::encode(&address));
        }
        cli::Command::SignTransaction(opt) => {
            let signature = sign_transaction(
                opt.wallet_id,
                &opt.hd_path,
                opt.chain_id,
                opt.nonce,
                opt.to,
                opt.value,
                opt.gas_price,
                opt.gas,
            )?;
            println!("Signature: {}", hex::encode(&signature));
        }
        cli::Command::Test => {
            tests::tests::test_workflow();
            println!("Tests passed");
        }
    }
    Ok(())
}
