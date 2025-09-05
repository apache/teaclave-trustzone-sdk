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

use crate::secure_storage_client::SecureStorageClient;
use crate::user::{Approver, System, TaUser, TxOperator};
use basic_utils::println;
use proto::{LatestWalletInfo, TaTransactionDetail};
use types::share::{
    ChainType, CkSignature, DeviceID, LegacyTransaction, MultiChainAccount, MultiChainTransaction,
    Secp256k1, TaApprovalChain, TaWalletInfo, TeeOnlineStatus, TransactionID, WalletID,
    SUPPORTED_CHAIN_TYPES,
};
use types::trust::{
    AccountXpriv, BtcAccountXpriv, CkChangeInfo, CkRecipientInfo, CkUtxoInfo, EthAccountXpriv,
    TeeDeviceInfo, TeeTransaction, TeeWallet, WalletSecret,
};

use anyhow::{bail, ensure, Result};
use std::collections::HashSet;
use std::convert::TryInto;

// manage all commands through TLS
pub struct StateManager {
    db_handler: SecureStorageClient,
    device_id: DeviceID,
    tee_status: TeeOnlineStatus,
    existing_wallet_ids: HashSet<WalletID>,
}

impl StateManager {
    pub fn init() -> Result<Self> {
        let db_handler = SecureStorageClient::init();
        let device_info = match db_handler
            .get::<String, TeeDeviceInfo>(&"tee_device_info".to_string())
        {
            Ok(device_info) => device_info,
            Err(_) => {
                bail!("[-] StateManager::init(): error: device info not found, device has to be initialized first")
            }
        };
        let tee_status = match db_handler.get::<String, TeeOnlineStatus>(&"tee_status".to_string())
        {
            Ok(tee_status) => tee_status,
            Err(_) => TeeOnlineStatus::init(),
        };
        let existing_wallet_ids = db_handler
            .list_entries::<WalletID, TeeWallet>()?
            .keys()
            .cloned()
            .collect();

        Ok(Self {
            db_handler,
            device_id: device_info.device_id,
            tee_status,
            existing_wallet_ids,
        })
    }

    pub fn set_wallets(
        &mut self,
        wallets: Vec<TaWalletInfo>,
        config_version: u64,
    ) -> Result<(Vec<LatestWalletInfo>, u64)> {
        // check if config_version is valid
        if self.tee_status.is_running() {
            ensure!(
                config_version > self.tee_status.config_version()?,
                "config_version should be greater than internal config_version"
            );
        }

        let wallet_ids_from_input: HashSet<WalletID> =
            wallets.iter().map(|w| w.wallet_id.clone()).collect();
        // ensure wallet_ids_from_input contains all existing_wallet_ids
        ensure!(
            self.existing_wallet_ids.is_subset(&wallet_ids_from_input),
            "wallets should contain all existing wallets"
        );

        for wallet_info in wallets {
            let wallet_id = wallet_info.wallet_id.clone();
            match self.load_wallet(&wallet_id) {
                // if wallet exist, update
                Ok(mut tee_wallet) => {
                    println!(
                        "[-] StateManager::set_wallets(): {:?} exist, update",
                        wallet_id
                    );
                    tee_wallet.update_approval_chain(wallet_info.approval_chain)?;
                    tee_wallet.update_authorized_operators(wallet_info.authorized_operators)?;
                    self.db_handler.put(&tee_wallet)?;
                }
                Err(_) => {
                    // if wallet doesn't exist, create
                    println!(
                        "[-] StateManager::set_wallets(): {:?} doesn't exist, create",
                        wallet_id
                    );
                    let _wallet = self.create_wallet(wallet_info)?;
                    self.existing_wallet_ids.insert(wallet_id);
                }
            }
        }

        self.tee_status.set_running(config_version);
        let wallets = self.list_wallets()?;
        Ok((wallets, config_version))
    }

    fn list_wallets(&self) -> Result<Vec<LatestWalletInfo>> {
        let mut latest_wallets = Vec::new();
        for wallet in self
            .db_handler
            .list_entries::<WalletID, TeeWallet>()?
            .values()
        {
            let wallet_info = LatestWalletInfo {
                wallet_id: wallet.id(),
                approval_chain: wallet.approval_chain_basic().clone(),
                authorized_operators: wallet.authorized_operators_basic().clone(),
                accounts: wallet.accounts().clone(),
            };
            latest_wallets.push(wallet_info);
        }
        Ok(latest_wallets)
    }

    pub fn create_tx(
        &self,
        _operator: &TxOperator,
        tx: MultiChainTransaction,
    ) -> Result<(TransactionID, TaApprovalChain)> {
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let from_wallet = self
            .db_handler
            .get::<WalletID, TeeWallet>(tx.from_wallet())?;
        ensure!(
            from_wallet.has_account(tx.from_account()),
            "account not found in wallet"
        );
        let tx_approval_chain: TaApprovalChain = from_wallet.approval_chain_basic().clone().into();
        let tee_tx = TeeTransaction::new(
            tx.hash()?,
            tx.from_wallet().clone(),
            tx_approval_chain.clone(),
            from_wallet.authorized_operators_basic().clone(),
        )?;
        self.db_handler.put(&tee_tx)?;
        Ok((tee_tx.id().clone(), tx_approval_chain))
    }

    pub fn approve_tx(
        &self,
        approver: &Approver,
        tx_id: TransactionID,
        current_approval_chain: &TaApprovalChain,
    ) -> Result<TaApprovalChain> {
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let mut tee_tx = self
            .db_handler
            .get::<TransactionID, TeeTransaction>(&tx_id)?;
        tee_tx.approve(approver.0.user_id(), current_approval_chain)?;
        self.db_handler.put(&tee_tx)?;
        Ok(tee_tx.get_approval_chain().clone())
    }

    pub fn reject_tx(
        &self,
        approver: &Approver,
        tx_id: TransactionID,
        current_approval_chain: &TaApprovalChain,
    ) -> Result<TaApprovalChain> {
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let mut tee_tx = self
            .db_handler
            .get::<TransactionID, TeeTransaction>(&tx_id)?;
        tee_tx.reject(approver.0.user_id(), current_approval_chain)?;
        self.db_handler.put(&tee_tx)?;
        Ok(tee_tx.get_approval_chain().clone())
    }

    pub fn recall_tx(
        &self,
        _operator: &TxOperator,
        tx_id: &TransactionID,
        current_approval_chain: &TaApprovalChain,
    ) -> Result<()> {
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let tee_tx = self
            .db_handler
            .get::<TransactionID, TeeTransaction>(tx_id)?;
        ensure!(
            tee_tx.is_approval_chain_up_to_date(current_approval_chain),
            "hash doesn't match"
        );
        self.db_handler
            .delete_entry::<TransactionID, TeeTransaction>(tx_id)?;
        println!("[-] StateManager::recall_tx(): {:?} recalled", tx_id);
        Ok(())
    }

    pub fn sign_tx(
        &self,
        user: &TaUser,
        tx_id: TransactionID,
        external_tx: MultiChainTransaction,
    ) -> Result<CkSignature> {
        // user should be operator or system
        ensure!(
            user.is_tx_operator() || user.is_system(),
            "user is not operator or system"
        );
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let tee_tx = self
            .db_handler
            .get::<TransactionID, TeeTransaction>(&tx_id)?;
        // tx should be ready for sign
        ensure!(tee_tx.is_ready_for_sign(), "tx is not ready for sign");
        // tx hash should match
        ensure!(
            tee_tx.hash_matches(&external_tx.hash()?),
            "tx hash doesn't match"
        );

        let account_xpriv = self
            .db_handler
            .get::<_, AccountXpriv>(external_tx.from_account())?;
        let signed_payload: CkSignature = self.process_sign(external_tx, account_xpriv)?;
        println!("[-] StateManager::sign_tx(): {:?} signed", tx_id);
        self.db_handler
            .delete_entry::<TransactionID, TeeTransaction>(&tx_id)?;

        Ok(signed_payload)
    }

    fn process_sign(
        &self,
        multichain_tx: MultiChainTransaction,
        xpriv: AccountXpriv,
    ) -> Result<CkSignature> {
        if let MultiChainTransaction::Eth(tx) = multichain_tx {
            let legacy_tx: LegacyTransaction = tx.try_into()?;
            let signed_payload = EthAccountXpriv(xpriv).sign_tx(&legacy_tx)?;
            Ok(signed_payload.into())
        } else if let MultiChainTransaction::Btc(tx) = multichain_tx {
            let secp = Secp256k1::new();
            let xpub = xpriv.xpub(&secp);
            let ck_utxos: Vec<CkUtxoInfo> = tx
                .utxo_list
                .into_iter()
                .map(|utxo| CkUtxoInfo::try_from(&xpub, utxo, &secp))
                .collect::<Result<Vec<CkUtxoInfo>>>()?;
            let ck_recipients: Vec<CkRecipientInfo> = tx
                .recipient_list
                .into_iter()
                .map(|recipient| recipient.try_into())
                .collect::<Result<Vec<CkRecipientInfo>>>()?;
            let ck_change = CkChangeInfo::try_from(&xpub, tx.change, &secp)?;

            BtcAccountXpriv(xpriv).sign_tx(ck_utxos, ck_recipients, ck_change, &secp)
        } else {
            bail!("unsupported chain type")
        }
    }

    pub fn list_tx(&self, _system: &System) -> Result<Vec<TaTransactionDetail>> {
        // check if tee is online
        ensure!(self.tee_status.is_running(), "TEE service is not running");

        let all_tee_tx = self
            .db_handler
            .list_entries::<TransactionID, TeeTransaction>()?;
        let mut tx_details = Vec::new();
        for tee_tx in all_tee_tx.values() {
            let tx_detail = TaTransactionDetail {
                tx_id: tee_tx.id().clone(),
                tx_hash: tee_tx.hash().clone(),
                from_wallet: tee_tx.from_wallet().clone(),
                latest_approval_chain: tee_tx.get_approval_chain().clone(),
                authorized_operators: tee_tx.authorized_operators().clone(),
            };
            tx_details.push(tx_detail);
        }
        Ok(tx_details)
    }

    pub fn get_tee_status(&self) -> TeeOnlineStatus {
        self.tee_status.clone()
    }

    pub fn get_device_id(&self) -> DeviceID {
        self.device_id.clone()
    }

    fn create_wallet(&mut self, wallet_info: TaWalletInfo) -> Result<TeeWallet> {
        let wallet_secret = WalletSecret::new(wallet_info.wallet_id.clone())?;
        let mut tee_wallet = TeeWallet::new(
            wallet_info.wallet_id.clone(),
            wallet_info.approval_chain,
            wallet_info.authorized_operators,
            &wallet_secret,
        )?;
        let accounts = self.add_account_inner(&mut tee_wallet, &SUPPORTED_CHAIN_TYPES)?;
        println!(
            "[-] StateManager::create_wallet(): {:?} created with accounts: {:?}",
            wallet_info.wallet_id, accounts
        );
        self.db_handler.put(&tee_wallet)?;
        Ok(tee_wallet)
    }

    fn load_wallet(&self, wallet_id: &WalletID) -> Result<TeeWallet> {
        self.db_handler.get::<WalletID, TeeWallet>(wallet_id)
    }

    fn _wallet_exist(&self, wallet_id: &WalletID) -> bool {
        self.existing_wallet_ids.contains(wallet_id)
    }

    fn add_account_inner(
        &mut self,
        tee_wallet: &mut TeeWallet,
        chain_types: &[ChainType],
    ) -> Result<Vec<MultiChainAccount>> {
        let mut multichain_accounts = Vec::new();
        for chain_type in chain_types {
            match chain_type {
                ChainType::Eth => {
                    let (account_xpriv, account_xpub) = tee_wallet.add_eth_account()?;
                    self.db_handler.put(&account_xpriv.0)?;
                    multichain_accounts.push(MultiChainAccount::Eth(account_xpub));
                }
                ChainType::Btc => {
                    let (account_xpriv, account_xpub) = tee_wallet.add_btc_account()?;
                    self.db_handler.put(&account_xpriv.0)?;
                    multichain_accounts.push(MultiChainAccount::Btc(account_xpub));
                }
            }
        }
        Ok(multichain_accounts)
    }
}
