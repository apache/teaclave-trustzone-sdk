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

use crate::{
    create_keypair, decrypt_data_by_rsa_keypair, encrypt_data_by_rsa_pubkey, verify_root_signature,
    SecureStorageClient,
};
use anyhow::{bail, ensure, Result};
use basic_utils::println;
use proto::{
    BackupWalletInput, BackupWalletOutput, InitBoardOutput, RestoreWalletInput, RestoreWalletOutput,
};
use secure_storage::{TeeKey, TeeKeyObject, TeeKeyUsage};
use std::convert::TryInto;
use types::share::{CkEncryptedPayload, DeviceID, TeeOnlineStatus, WalletID};
use types::trust::{TeeDeviceInfo, TeeWallet, TeeWalletForBackup};

pub fn init_board() -> Result<InitBoardOutput> {
    let db_client = SecureStorageClient::init();
    let signing_pubkey = create_keypair(&TeeKeyUsage::Signing, &db_client).unwrap();
    let device_id: DeviceID = signing_pubkey.clone().into();
    let tee_device_info = TeeDeviceInfo::new(device_id.clone());
    db_client.put(&tee_device_info)?;

    let backup_pubkey = create_keypair(&TeeKeyUsage::Backup, &db_client).unwrap();
    let output = InitBoardOutput {
        device_id,
        signing_pubkey,
        backup_pubkey,
    };
    Ok(output)
}

pub fn backup_wallet(input: BackupWalletInput) -> Result<BackupWalletOutput> {
    // verify signature
    ensure!(
        input.signature.is_some(),
        "[-] backup_wallet: signature is missing"
    );
    verify_root_signature(&input.serialize()?, &input.signature.unwrap())?;
    println!("[+] backup_wallet: signature verified");

    // load each wallet
    let mut backup_wallets: Vec<TeeWalletForBackup> = vec![];
    let db_client = SecureStorageClient::init();

    // if input.target_wallets is empty, backup all wallets
    let target_wallets = if input.target_wallets.is_empty() {
        println!("[+] backup_wallet: target_wallets is empty, backing up all wallets");
        let all_wallets: Vec<WalletID> = db_client
            .list_entries::<WalletID, TeeWallet>()?
            .keys()
            .cloned()
            .collect();
        all_wallets
    } else {
        println!(
            "[+] backup_wallet: target_wallets: {:?}",
            input.target_wallets
        );
        input.target_wallets.iter().cloned().collect()
    };

    for wallet_id in &target_wallets {
        println!("[+] backup_wallet: wallet_id: {:?}", wallet_id);
        let wallet: TeeWalletForBackup = db_client.get::<_, TeeWallet>(wallet_id)?.try_into()?;
        // print the checksum of each priv key for double check
        println!(
            "[+] backup_wallet: wallet_id: {:?}, priv_key checksum: {:?}",
            wallet_id,
            wallet.xpriv_checksum()
        );
        backup_wallets.push(wallet);
    }

    // encrypt serialized Vec<TeeWallet>
    let plain_data = bincode::serialize(&backup_wallets)?;
    println!("[+] backup_wallet: plain_data len: {:?}", plain_data.len());
    let encrypted_payload: CkEncryptedPayload =
        encrypt_data_by_rsa_pubkey(plain_data, input.backup_to_device_pubkey)?;
    let output = BackupWalletOutput {
        backup_to_device: input.backup_to_device_id,
        wallet_ids: backup_wallets.iter().map(|w| w.id()).collect(),
        encrypted: encrypted_payload,
    };
    Ok(output)
}

pub fn restore_wallet(input: RestoreWalletInput) -> Result<RestoreWalletOutput> {
    // check destination device
    let db_client = SecureStorageClient::init();
    let tee_device_info: TeeDeviceInfo =
        db_client.get::<_, TeeDeviceInfo>(&"tee_device_info".to_string())?;
    ensure!(
        tee_device_info.device_id == input.backup_to_device,
        "[-] restore_wallet: destination device mismatch: current {:?} != input {:?}",
        tee_device_info.device_id,
        input.backup_to_device
    );

    // load device backup key
    let backup_key = db_client.get::<String, TeeKey>(&TeeKeyUsage::Backup.into())?;
    let rsa_keypair = match backup_key.object() {
        TeeKeyObject::RsaKeyPair(rsa_keypair) => rsa_keypair,
        _ => bail!("[-] restore_wallet: invalid key object"),
    };
    let decrypted_data = decrypt_data_by_rsa_keypair(input.encrypted, rsa_keypair)?;
    let backup_wallets: Vec<TeeWalletForBackup> = bincode::deserialize(&decrypted_data)?;
    let restore_wallet_ids = backup_wallets.iter().map(|w| w.id()).collect();
    // check if wallet ids same as input
    ensure!(
        restore_wallet_ids == input.wallet_ids,
        "[-] restore_wallet: wallet id mismatch"
    );

    // restore each wallet
    for wallet in backup_wallets {
        println!(
            "[+] restore_wallet: wallet_id: {:?}, priv_key checksum: {:?}",
            wallet.id(),
            wallet.xpriv_checksum()
        );
        let mut tee_wallet: TeeWallet = wallet.try_into()?;
        db_client.put::<_, TeeWallet>(&tee_wallet)?;

        let all_account_xpriv = tee_wallet.derive_all_xpriv_for_restore()?;
        for account_xpriv in all_account_xpriv {
            db_client.put(&account_xpriv)?;
        }
        println!("[-] StateManager::init(): restore all AccountXpriv");

        print!("Restored wallet: {:?}", tee_wallet.id());
    }

    let output = RestoreWalletOutput {
        wallet_ids: restore_wallet_ids,
    };
    Ok(output)
}

pub fn clear_wallet_storage() -> Result<()> {
    let db_client = SecureStorageClient::init();
    // list all TeeWallet
    let wallet_ids: Vec<WalletID> = db_client
        .list_entries::<WalletID, TeeWallet>()?
        .keys()
        .cloned()
        .collect();
    println!("[+] clear_wallet_storage: wallet_ids: {:?}", wallet_ids);
    // delete all TeeWallet
    for wallet_id in wallet_ids {
        db_client.delete_entry::<WalletID, TeeWallet>(&wallet_id)?;
    }
    println!("[+] clear_wallet_storage: all wallets deleted");
    // update TeeOnlineStatus
    let tee_online_status = TeeOnlineStatus::WaitingForSync;
    db_client.put(&tee_online_status)?;
    println!("[+] clear_wallet_storage: TeeOnlineStatus updated to WaitingForSync");

    Ok(())
}
