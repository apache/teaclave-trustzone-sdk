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

use anyhow::{bail, ensure, Result};
use core::{
    backup_wallet, init_board, restore_wallet, Approver, InterCertConfig, StateManager, System,
    TaUser, TlsContext, TlsSessionManager, TxOperator, UserRegistry,
};
use lazy_static::lazy_static;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{Error, ErrorKind, Parameters};
use proto::{
    ApproveTransactionInput, ApproveTransactionOutput, BackupWalletInput, CreateTransactionInput,
    CreateTransactionOutput, GetTeeStatusOutput, ListPendingTransactionInput,
    ListPendingTransactionOutput, RecallTransactionInput, RecallTransactionOutput,
    RestoreWalletInput, SignTransactionInput, SignTransactionOutput, SyncWithTeeInput,
    SyncWithTeeOutput, TaCommand, TlsCommandRequest,
};
use serde::Serialize;
use std::convert::TryInto;
use std::io::Write;
use std::sync::RwLock;
use types::share::{ApprovalOperation, CkHash};

lazy_static! {
    static ref TLS_SESSION_MANAGER: RwLock<TlsSessionManager> =
        RwLock::new(TlsSessionManager::init());
    static ref STATE_MANAGER: RwLock<StateManager> = RwLock::new(StateManager::init().unwrap());
    static ref USER_REGISTRY: RwLock<UserRegistry> = RwLock::new(UserRegistry::init());
}

#[ta_create]
fn create() -> optee_utee::Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> optee_utee::Result<()> {
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
fn invoke_command(cmd_id: u32, mut params: &mut Parameters) -> optee_utee::Result<()> {
    trace_println!("[+] TA invoke command");
    let session_id = unsafe { params.0.as_value().unwrap().a() };
    trace_println!("[+] session id: {}, command id : {}", session_id, cmd_id);

    match dispatch(cmd_id, session_id, &mut params) {
        Ok(_) => Ok(()),
        Err(e) => {
            trace_println!("[-] tls command error: {}", e);
            Err(Error::new(ErrorKind::BadParameters))
        }
    }
}

fn dispatch(cmd_id: u32, session_id: u32, mut params: &mut Parameters) -> Result<()> {
    let mut tls_session_manager = TLS_SESSION_MANAGER.write().unwrap();
    match TaCommand::from(cmd_id) {
        // setup board: should be triggered by CA
        TaCommand::InitBoard => {
            trace_println!("[+] init_board");
            let output = init_board()?;
            restore_output(&mut params, output)
        }
        TaCommand::BackupWallet => {
            let mut p1 = unsafe { params.1.as_memref().unwrap() };
            let buffer = p1.buffer();
            let input: BackupWalletInput = bincode::deserialize(&buffer)?;
            let output = backup_wallet(input)?;
            restore_output(&mut params, output)
        }
        TaCommand::RestoreWallet => {
            let mut p1 = unsafe { params.1.as_memref().unwrap() };
            let buffer = p1.buffer();
            let input: RestoreWalletInput = bincode::deserialize(&buffer)?;
            let output = restore_wallet(input)?;
            restore_output(&mut params, output)
        }
        #[cfg(feature = "enable_clear_storage")]
        TaCommand::ClearWalletStorage => {
            trace_println!("[+] clear_wallet_storage");
            clear_wallet_storage()?;
            Ok(())
        }
        // set tee_inter_cert, and start the TA server for listening
        TaCommand::StartServer => {
            trace_println!("[+] start_server");
            let mut p1 = unsafe { params.1.as_memref().unwrap() };
            let tee_inter_cert = p1.buffer().to_vec();
            let inter_cert_config = InterCertConfig::new(
                tee_inter_cert.clone(),
                "CK ECDSA level 2 intermediate".to_string(),
                "testserver.com".to_string(),
            );
            tls_session_manager.init_tls_server(inter_cert_config)
        }
        // setup tls session, create a server connection of specific session id
        TaCommand::NewTlsSession => {
            trace_println!("[+] new_tls_session");
            let user_registery = USER_REGISTRY.read().unwrap();
            let accepted_pubkey_hash = user_registery.get_accepted_pubkey_hash();
            tls_session_manager.new_tls_session(session_id, accepted_pubkey_hash)
        }
        // handshake and receive request
        TaCommand::DoTlsRead => {
            let mut p1 = unsafe { params.1.as_memref().unwrap() };
            let buffer = p1.buffer();
            trace_println!("[+] do_tls_read, buf len: {}", buffer.len());
            let request = tls_session_manager.do_tls_read(session_id, buffer)?;
            if !request.is_empty() {
                let tls_context = tls_session_manager.construct_tls_context(session_id)?;
                let command_request: TlsCommandRequest = bincode::deserialize(&request)?;
                trace_println!("[+] command_request: {:?}", command_request.command);
                let response = match process_inner_command(tls_context, command_request) {
                    Ok(response) => response,
                    Err(e) => {
                        trace_println!("[-] process_inner_command error: {}", e);
                        format!("TEE Error: {:?}", e).as_bytes().to_vec()
                    }
                };
                trace_println!("[+] process_command done");
                tls_session_manager.write_response(session_id, &response)?;
            } else {
                trace_println!("[+] request is empty");
            }
            Ok(())
        }
        TaCommand::DoTlsWrite => {
            trace_println!("[+] do_tls_write");
            let mut p1 = unsafe { params.1.as_memref().unwrap() };
            let mut p2 = unsafe { params.2.as_value().unwrap() };
            let mut buffer = p1.buffer();
            let n = tls_session_manager.do_tls_write(session_id, &mut buffer)?;
            p2.set_a(n as u32);
            Ok(())
        }
        TaCommand::CloseTlsSession => {
            trace_println!("[+] close_tls_session");
            tls_session_manager.close_tls_session(session_id)
        }
        _ => bail!("Unsupported command"),
    }
}

fn restore_output<T>(params: &mut Parameters, output: T) -> Result<()>
where
    T: Serialize,
{
    let output = bincode::serialize(&output)?;
    let mut p2 = unsafe { params.2.as_memref()? };
    let mut p3 = unsafe { params.3.as_value()? };
    p2.buffer().write(&output)?;
    p3.set_a(output.len() as u32);
    trace_println!("[+] restore_output: finished");

    Ok(())
}

// processing commands which invoked through TLS
fn process_inner_command(context: TlsContext, request: TlsCommandRequest) -> Result<Vec<u8>> {
    let mut state_manager = STATE_MANAGER.write().unwrap();
    let mut user_registry = USER_REGISTRY.write().unwrap();
    let client_pubkey_hash: CkHash = context.try_into()?;

    let output = match request.command {
        // wallet commands:
        TaCommand::SyncWithTee => {
            let input: SyncWithTeeInput = bincode::deserialize(&request.request)?;
            let config = input.signed_config;
            ensure!(config.signature.is_some(), "signature is missing");
            let _system: System = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] SyncWithTee: system authenticated");

            user_registry.set_users(config.user_registry);
            let (latest_wallets, config_version) =
                state_manager.set_wallets(config.wallets, config.config_version)?;
            let output = SyncWithTeeOutput {
                latest_wallets,
                config_version,
            };
            bincode::serialize(&output)?
        }
        // transaction commands:
        TaCommand::CreateTransaction => {
            let input: CreateTransactionInput = bincode::deserialize(&request.request)?;
            let operator: TxOperator = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] CreateTransaction: operator authenticated");
            // create tx
            let (tx_id, latest_approval_chain) = state_manager.create_tx(&operator, input.tx)?;
            bincode::serialize(&CreateTransactionOutput {
                tx_id,
                latest_approval_chain,
            })?
        }
        TaCommand::ApproveTransaction => {
            let input: ApproveTransactionInput = bincode::deserialize(&request.request)?;
            let approver: Approver = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] ApproveTransaction: approver authenticated");
            let latest_approval_chain = match input.operation {
                ApprovalOperation::Approve => state_manager.approve_tx(
                    &approver,
                    input.tx_id,
                    &input.current_approval_chain,
                )?,
                ApprovalOperation::Reject => state_manager.reject_tx(
                    &approver,
                    input.tx_id,
                    &input.current_approval_chain,
                )?,
            };
            bincode::serialize(&ApproveTransactionOutput {
                latest_approval_chain,
            })?
        }
        TaCommand::RecallTransaction => {
            let input: RecallTransactionInput = bincode::deserialize(&request.request)?;
            let operator: TxOperator = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] RecallTransaction: operator authenticated");
            state_manager.recall_tx(&operator, &input.tx_id, &input.current_approval_chain)?;
            bincode::serialize(&RecallTransactionOutput { tx_id: input.tx_id })?
        }
        TaCommand::SignTransaction => {
            let input: SignTransactionInput = bincode::deserialize(&request.request)?;
            // Operator or System can sign tx
            let user: TaUser = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] SignTransaction: user authenticated");
            let signed_tx = state_manager.sign_tx(&user, input.tx_id, input.tx)?;
            bincode::serialize(&SignTransactionOutput { signed_tx })?
        }
        TaCommand::ListPendingTransaction => {
            let _input: ListPendingTransactionInput = bincode::deserialize(&request.request)?;
            let system: System = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] ListPendingTransaction: system authenticated");
            let tx_details = state_manager.list_tx(&system)?;
            bincode::serialize(&ListPendingTransactionOutput { tx_details })?
        }
        // tee status command
        TaCommand::GetTeeStatus => {
            let _system: System = user_registry.auth_as_role(&client_pubkey_hash)?;
            trace_println!("[+] GetTeeStatus: system authenticated");
            let device_id = state_manager.get_device_id();
            let tee_status = state_manager.get_tee_status();
            bincode::serialize(&GetTeeStatusOutput {
                device_id,
                tee_status,
            })?
        }
        _ => bail!("Unsupported command"),
    };
    Ok(output)
}

// TA configurations
const TA_FLAGS: u32 = 0;
const TA_DATA_SIZE: u32 = 4 * 1024 * 1024;
const TA_STACK_SIZE: u32 = 512 * 1024;
const TA_VERSION: &[u8] = b"0.2\0";
const TA_DESCRIPTION: &[u8] = b"This is a tls server example.\0";
const EXT_PROP_VALUE_1: &[u8] = b"TLS Server TA\0";
const EXT_PROP_VALUE_2: u32 = 0x0010;
const TRACE_LEVEL: i32 = 4;
const TRACE_EXT_PREFIX: &[u8] = b"TA\0";
const TA_FRAMEWORK_STACK_SIZE: u32 = 2048;

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
