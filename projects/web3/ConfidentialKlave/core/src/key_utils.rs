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

use std::convert::{TryFrom, TryInto};

use crate::{secure_storage_client::SecureStorageClient, ROOT_PUBKEY};
use anyhow::{bail, Result};
use basic_utils::println;
use crypto::{verify_signature_p384, AesGcm128Key, EcdsaKeyPair};
use secure_storage::{
    EcdsaKeyPairBytes, RsaKeyObject, RsaKeyPair, RsaPublicKey, TeeKey, TeeKeyObject, TeeKeyUsage,
};
use types::share::{CkEncryptedPayload, CkPublicKey, CkSignature};

const RSA_KEY_SIZE: usize = 2048;

pub fn create_keypair(usage: &TeeKeyUsage, db_client: &SecureStorageClient) -> Result<CkPublicKey> {
    if let Ok(tee_key) = db_client.get::<String, TeeKey>(&usage.into()) {
        println!("[-] create_keypair: keypair already exists, skipping creation");
        return tee_key.try_into();
    }
    match usage {
        TeeKeyUsage::Backup => {
            println!("[+] create_keypair: creating backup keypair...");
            let (key, pub_key) = create_backup_keypair(RSA_KEY_SIZE)?;
            db_client.put(&key)?;
            Ok(pub_key)
        }
        TeeKeyUsage::Signing => {
            println!("[+] create_keypair: creating signing keypair...");
            let (key, pub_key) = create_signing_keypair()?;
            db_client.put(&key)?;
            Ok(pub_key)
        }
    }
}

fn create_backup_keypair(key_size: usize) -> Result<(TeeKey, CkPublicKey)> {
    let mut kp_object = RsaKeyObject::allocate_keypair_object(key_size)?;
    kp_object.generate_key()?;
    let kp: RsaKeyPair = kp_object.try_into()?;
    let pubkey = kp.export_public_key()?;
    println!("[+] create_rsa_keypair: pubkey: {:?}", &pubkey);
    let key = TeeKey::new(TeeKeyUsage::Backup, TeeKeyObject::RsaKeyPair(kp))?;
    Ok((key, pubkey))
}

fn create_signing_keypair() -> Result<(TeeKey, CkPublicKey)> {
    let ecdsa_keypair = EcdsaKeyPair::new()?;
    let key = TeeKey::new(
        TeeKeyUsage::Signing,
        TeeKeyObject::EcdsaKeyPair(EcdsaKeyPairBytes {
            pkcs8_bytes: ecdsa_keypair.prv_key().to_vec(),
        }),
    )?;
    Ok((key, CkPublicKey::from(ecdsa_keypair.pub_key().to_vec())))
}

pub fn load_signing_keypair(db_client: &SecureStorageClient) -> Result<EcdsaKeyPair> {
    let key: TeeKey = db_client.get::<String, TeeKey>(&TeeKeyUsage::Signing.into())?;
    match key.object() {
        TeeKeyObject::EcdsaKeyPair(ecdsa_key_pair_bytes) => ecdsa_key_pair_bytes.try_into(),
        _ => bail!("[-] load_inter_keypair_from_secure_storage: invalid key object"),
    }
}

pub fn load_backup_keypair(db_client: &SecureStorageClient) -> Result<RsaKeyPair> {
    let key: TeeKey = db_client.get::<String, TeeKey>(&TeeKeyUsage::Backup.into())?;
    match key.object() {
        TeeKeyObject::RsaKeyPair(rsa_key_pair) => Ok(rsa_key_pair),
        _ => bail!("[-] load_backup_keypair_from_secure_storage: invalid key object"),
    }
}

pub fn verify_root_signature(data: &[u8], signature: &CkSignature) -> Result<()> {
    verify_signature_p384(ROOT_PUBKEY, data, signature.as_bytes())
}

pub fn encrypt_data_by_rsa_pubkey(
    data: Vec<u8>,
    pubkey: CkPublicKey,
) -> Result<CkEncryptedPayload> {
    // generate random data key
    let data_key = AesGcm128Key::generate()?;
    let mut data_in_out = data.clone();
    println!("[+] generate data_key");
    // encrypt data with data key
    let _cmac = data_key.encrypt(&mut data_in_out)?;
    println!("[+] encrypted_data_len: {:?}", &data_in_out.len());
    let serialized_data_key = bincode::serialize(&data_key)?;

    // use rsa public key to encrypt data key
    let public_key = RsaPublicKey::try_from(pubkey)?;
    // allocate public key object for OP-TEE
    let mut rsa_public_key_object: RsaKeyObject = public_key.try_into()?;
    let encrypted_data_key = rsa_public_key_object.encrypt(&serialized_data_key)?;
    println!(
        "[+] encrypted_data_key len : {:?}",
        &encrypted_data_key.len()
    );

    Ok(CkEncryptedPayload {
        encrypted_data: data_in_out,
        encrypted_data_key,
    })
}

pub fn decrypt_data_by_rsa_keypair(
    encrypted_payload: CkEncryptedPayload,
    keypair: RsaKeyPair,
) -> Result<Vec<u8>> {
    let mut key_object: RsaKeyObject = keypair.try_into()?;
    // decrypt data key
    let serialized_data_key = key_object.decrypt(&encrypted_payload.encrypted_data_key)?;
    let data_key: AesGcm128Key = bincode::deserialize(&serialized_data_key)?;
    println!("[+] decrypt data_key");

    // decrypt data
    let mut data_in_out = encrypted_payload.encrypted_data.clone();
    let _cmac = data_key.decrypt(&mut data_in_out)?;
    println!("[+] decrypted_data_len: {:?}", &data_in_out.len());
    Ok(data_in_out)
}
