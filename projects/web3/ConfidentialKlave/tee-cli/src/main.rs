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

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use constants::{OUTPUT_MAX_SIZE, TLS_UUID};
use optee_teec::{Context, Operation, ParamType, Uuid};
use optee_teec::{ParamTmpRef, ParamValue};
use proto::{BackupWalletOutput, InitBoardOutput, RestoreWalletOutput, TaCommand};
use std::fs;
use structopt::StructOpt;
use types::share::DeviceID;

fn invoke_command(command: TaCommand, input: &[u8]) -> optee_teec::Result<Vec<u8>> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(TLS_UUID)
        .map_err(|_| optee_teec::Error::new(optee_teec::ErrorKind::ItemNotFound))?;
    let mut session = ctx.open_session(uuid)?;

    println!("CA: command: {:?}", command);
    // reserved for tls session id
    let p0 = ParamValue::new(0, 0, ParamType::ValueInput);
    // input buffer
    let p1 = ParamTmpRef::new_input(input);

    // output buffer
    let mut output = vec![0u8; OUTPUT_MAX_SIZE];
    let p2 = ParamTmpRef::new_output(output.as_mut_slice());
    // output buffer size
    let p3 = ParamValue::new(0, 0, ParamType::ValueInout);

    let mut operation = Operation::new(0, p0, p1, p2, p3);
    session.invoke_command(command as u32, &mut operation)?;

    let output_len = operation.parameters().3.a() as usize;
    Ok(output[..output_len].to_vec())
}

fn main() -> Result<()> {
    let args = cli::Opt::from_args();
    match args.command {
        cli::Command::InitBoard(_opt) => {
            let serialized_output = invoke_command(TaCommand::InitBoard, &[0u8])?;
            let output: InitBoardOutput = bincode::deserialize(&serialized_output)?;
            let device_id: DeviceID = output.signing_pubkey.into();
            assert_eq!(device_id, output.device_id);
            let device_id_string: String = output.device_id.into();
            let encoded_pubkeys = general_purpose::STANDARD.encode(&serialized_output);
            let output_file_name = device_id_string + ".pubkeys";
            fs::write(&output_file_name, &encoded_pubkeys)?;
            println!("Encoded Public keys: \n{}", &encoded_pubkeys);
            println!("Encoded Public keys written to {}", output_file_name);
        }
        cli::Command::BackupWallet(opt) => {
            let signed_backup_list = fs::read(&opt.signed_backup_list)?; // serialized BackupWalletInput
            let serialized_output = invoke_command(TaCommand::BackupWallet, &signed_backup_list)?;
            let output: BackupWalletOutput = bincode::deserialize(&serialized_output)?;

            let device_id: String = output.backup_to_device.clone().into();
            let data_file_name = device_id + ".encrypted";
            // write serialized EncryptedDataOutput to file
            fs::write(&data_file_name, serialized_output)?;
            println!("Encrypted data written to {}", data_file_name);
        }
        cli::Command::RestoreWallet(opt) => {
            let encrypted_data = fs::read(&opt.encrypted_data)?;
            let serialized_output = invoke_command(TaCommand::RestoreWallet, &encrypted_data)?;
            let output: RestoreWalletOutput = bincode::deserialize(&serialized_output)?;
            println!("Restored wallets: {:?}", output.wallet_ids);
        }
        cli::Command::ClearWalletStorage(_opt) => {
            let _ = invoke_command(TaCommand::ClearWalletStorage, &[0u8])?;
            println!("ClearWalletStorage finished");
        }
    }
    Ok(())
}
