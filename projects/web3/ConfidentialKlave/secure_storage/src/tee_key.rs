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
use crypto::{EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use types::{share::CkPublicKey, Storable};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TeeKeyUsage {
    Backup,
    Signing,
}

impl From<TeeKeyUsage> for String {
    fn from(usage: TeeKeyUsage) -> String {
        (&usage).into()
    }
}

impl From<&TeeKeyUsage> for String {
    fn from(usage: &TeeKeyUsage) -> String {
        match usage {
            TeeKeyUsage::Backup => "keystore-backup".to_string(),
            TeeKeyUsage::Signing => "keystore-signing".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeeKey {
    usage: TeeKeyUsage, // as storage key
    object: TeeKeyObject,
}

impl TeeKey {
    pub fn new(usage: TeeKeyUsage, object: TeeKeyObject) -> Result<Self> {
        Ok(Self { usage, object })
    }

    pub fn object(&self) -> TeeKeyObject {
        self.object.clone()
    }
}

impl Storable<String> for TeeKey {
    fn unique_id(&self) -> String {
        self.usage.clone().into()
    }
}

impl TryFrom<TeeKey> for CkPublicKey {
    type Error = anyhow::Error;

    fn try_from(tee_key: TeeKey) -> Result<Self> {
        match tee_key.object {
            TeeKeyObject::RsaKeyPair(key_pair) => key_pair.export_public_key(),
            TeeKeyObject::RsaPublicKey(public_key) => public_key.clone().try_into(),
            TeeKeyObject::EcdsaKeyPair(ecdsa_key_pair_bytes) => {
                let kp = EcdsaKeyPair::try_from(ecdsa_key_pair_bytes)?;
                Ok(CkPublicKey::from(kp.pub_key().to_vec()))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TeeKeyObject {
    RsaKeyPair(RsaKeyPair),          // for encrypting backup data key
    RsaPublicKey(RsaPublicKey),      // for decrypting backup data key
    EcdsaKeyPair(EcdsaKeyPairBytes), // for signing TLS end certificate, as intermediate CA
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EcdsaKeyPairBytes {
    pub pkcs8_bytes: Vec<u8>,
}

impl TryFrom<EcdsaKeyPairBytes> for EcdsaKeyPair {
    type Error = anyhow::Error;

    fn try_from(ecdsa_key_pair_bytes: EcdsaKeyPairBytes) -> Result<Self> {
        let key_pair = EcdsaKeyPair::from_bytes(
            &ecdsa_key_pair_bytes.pkcs8_bytes,
            &ECDSA_P256_SHA256_ASN1_SIGNING,
        )?;
        Ok(key_pair)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsaKeyPair {
    key_size: usize, // The number of bits in the modulus. can be 256, ..., 4096
    public_key: RsaPublicKey,
    private_key: RsaPrivateKey,
}

impl RsaKeyPair {
    pub fn new(key_size: usize, public_key: RsaPublicKey, private_key: RsaPrivateKey) -> Self {
        Self {
            key_size,
            public_key,
            private_key,
        }
    }

    pub fn export_public_key(&self) -> Result<CkPublicKey> {
        self.public_key.clone().try_into()
    }

    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }

    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private_key
    }

    pub fn key_size(&self) -> usize {
        self.key_size
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsaPrivateKey {
    exponent: Vec<u8>,
}

impl RsaPrivateKey {
    pub fn new(e: &[u8]) -> Result<Self> {
        Ok(Self {
            exponent: e.to_vec(),
        })
    }

    pub fn exponent(&self) -> &[u8] {
        &self.exponent
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RsaPublicKey {
    key_size: usize,
    modulus: Vec<u8>,
    exponent: Vec<u8>,
}

impl RsaPublicKey {
    pub fn new(key_size: usize, m: &[u8], e: &[u8]) -> Result<Self> {
        Ok(Self {
            key_size,
            modulus: m.to_vec(),
            exponent: e.to_vec(),
        })
    }

    pub fn key_size(&self) -> usize {
        self.key_size
    }

    pub fn modulus(&self) -> &[u8] {
        &self.modulus
    }

    pub fn exponent(&self) -> &[u8] {
        &self.exponent
    }
}

impl TryFrom<RsaPublicKey> for CkPublicKey {
    type Error = anyhow::Error;

    fn try_from(rsa_public_key: RsaPublicKey) -> Result<Self> {
        // traditional RSA public key format: modulus + exponent
        // we add a 4-byte prefix to indicate the key size
        let bytes = bincode::serialize(&rsa_public_key).map_err(|e| anyhow!(e))?;
        Ok(CkPublicKey::from(bytes))
    }
}

impl TryFrom<CkPublicKey> for RsaPublicKey {
    type Error = anyhow::Error;

    fn try_from(ck_public_key: CkPublicKey) -> Result<Self> {
        bincode::deserialize(ck_public_key.as_bytes()).map_err(|e| anyhow!(e))
    }
}
