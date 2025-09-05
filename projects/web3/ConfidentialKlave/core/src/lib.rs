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

mod key_utils;
pub use key_utils::*;
mod secure_storage_client;
pub use secure_storage_client::SecureStorageClient;
mod state_manager;
pub use state_manager::StateManager;
mod tls;
pub use tls::handler_context::{InterCertConfig, TlsContext};
pub use tls::session_manager::TlsSessionManager;
mod user;
pub use user::*;
mod user_registry;
pub use user_registry::UserRegistry;
mod device_cmd_handler;
pub use device_cmd_handler::*;

const ROOT_PUBKEY: &[u8] = include_bytes!("../../pubkeys/ca.pub");
const SYSTEM_PUBKEY: &[u8] = include_bytes!("../../pubkeys/system.pub");

#[cfg(feature = "ta_unit_test")]
pub mod tests {
    use super::*;
    use test_utils::check_all_passed;

    pub fn run_tests() -> bool {
        trace_println!("[+] core::run_tests");
        check_all_passed!(
            wallet::tests::run_tests(),
            eth::tests::run_tests(),
            twallet::tests::run_tests()
        )
    }
}

#[cfg(not(target_os = "optee"))]
pub mod tests {
    use std::{convert::TryInto, sync::RwLock};

    use anyhow::{anyhow, ensure, Result};
    use proto::{
        CreateTransactionInput, CreateTransactionOutput, InitBoardOutput, SyncWithTeeInput,
        SyncWithTeeOutput, TaCommand, TlsCommandRequest,
    };
    use secure_storage::TeeKeyUsage;
    use types::share::{
        CkHash, DeviceID, EthTransaction, MultiChainTransaction, TeeConfig, WalletID,
    };

    use crate::{
        key_utils::create_keypair, secure_storage_client::SecureStorageClient,
        state_manager::StateManager, tls::handler_context::TlsContext, user::TxOperator,
        user_registry::UserRegistry, verify_root_signature, ROOT_PUBKEY, SYSTEM_PUBKEY,
    };
    use lazy_static::lazy_static;
    lazy_static! {
        static ref STATE_MANAGER: RwLock<StateManager> = RwLock::new(StateManager::init().unwrap());
        static ref USER_REGISTRY: RwLock<UserRegistry> = RwLock::new(UserRegistry::init());
    }

    #[test]
    fn test_ta_workflow() {
        // init board: triggered by local CA
        let db_client = SecureStorageClient::init();
        let signing_pubkey = create_keypair(&TeeKeyUsage::Signing, &db_client).unwrap();
        let device_id: DeviceID = signing_pubkey.clone().into();
        let backup_pubkey = create_keypair(&TeeKeyUsage::Backup, &db_client).unwrap();
        let output = InitBoardOutput {
            device_id,
            signing_pubkey,
            backup_pubkey,
        };
        // register device on authority and get the tee certificate

        // start server: triggered by local CA
        // init TlsSessionManager with the tee certificate
        // start listening

        // test sync to tee
        // prepare input out of tee
        let input = SyncWithTeeInput {
            signed_config: TeeConfig {
                user_registry: Vec::new(),
                wallets: vec![],
                config_version: 0,
                signature: None,
            },
        };
        let request = TlsCommandRequest {
            command: TaCommand::SyncWithTee,
            request: bincode::serialize(&input).unwrap(),
        };
        // process cmd in tee
        let mocked_tls_context = TlsContext::new(SYSTEM_PUBKEY.to_vec());
        process_tls_cmd(mocked_tls_context, request).unwrap();

        // test create transaction
        // prepare input out of tee
        let tx = MultiChainTransaction::Eth(EthTransaction {
            chain: 1,
            nonce: None,
            gas_price: 0,
            gas: 0,
            from_wallet: WalletID::new().unwrap(),
            from_account: [0u8; 20].into(),
            to: [0u8; 20],
            value: 0,
            data: vec![],
        });
        let input = CreateTransactionInput { tx };
        let request = TlsCommandRequest {
            command: TaCommand::CreateTransaction,
            request: bincode::serialize(&input).unwrap(),
        };
        // process cmd in tee
        let mocked_tls_context = TlsContext::new(SYSTEM_PUBKEY.clone().to_vec());
        process_tls_cmd(mocked_tls_context, request).unwrap();
    }

    fn process_tls_cmd(context: TlsContext, request: TlsCommandRequest) -> Result<()> {
        let mut state_manager = STATE_MANAGER.write().unwrap();
        let mut user_registry = USER_REGISTRY.write().unwrap();

        match request.command {
            TaCommand::SyncWithTee => {
                let input: SyncWithTeeInput = bincode::deserialize(&request.request)?;
                let config = input.signed_config;
                ensure!(config.signature.is_some(), "signature is missing");
                verify_root_signature(&config.serialize()?, &config.signature.unwrap())?;

                user_registry.set_users(config.user_registry);
                let (latest_wallets, config_version) =
                    state_manager.set_wallets(config.wallets, config.config_version)?;
                let output = SyncWithTeeOutput {
                    latest_wallets,
                    config_version,
                };
            }
            TaCommand::CreateTransaction => {
                let input: CreateTransactionInput = bincode::deserialize(&request.request)?;
                // verify tls client
                let client_pubkey_hash: CkHash = context.try_into()?;
                let _operator: TxOperator = user_registry.auth_as_role(&client_pubkey_hash)?;
                // create tx
                let (tx_id, latest_approval_chain) =
                    state_manager.create_tx(&_operator, input.tx)?;
                let output = CreateTransactionOutput {
                    tx_id,
                    latest_approval_chain,
                };
            }
            _ => {
                unimplemented!()
            }
        };
        Ok(())
    }
}
