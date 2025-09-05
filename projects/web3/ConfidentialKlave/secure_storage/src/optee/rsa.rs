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

use crate::{RsaKeyPair, RsaPrivateKey, RsaPublicKey};
use anyhow::{anyhow, bail, ensure, Result};
use basic_utils::println;
use optee_utee::{AlgorithmId, Asymmetric, OperationMode};
pub use optee_utee::{
    Attribute, AttributeId, AttributeMemref, TransientObject, TransientObjectType,
};
use std::convert::{TryFrom, TryInto};

// the wrapper of optee_utee::TransientObject
pub struct RsaKeyObject {
    key_size: usize, // The number of bits in the modulus. can be 256, ..., 4096
    key_object: TransientObject,
    _key_object_type: TransientObjectType,
}
impl RsaKeyObject {
    pub fn allocate_keypair_object(key_size: usize) -> Result<Self> {
        let object = TransientObject::allocate(TransientObjectType::RsaKeypair, key_size)
            .map_err(|e| anyhow!(e))?;
        Ok(Self {
            key_size,
            key_object: object,
            _key_object_type: TransientObjectType::RsaKeypair,
        })
    }
    pub fn allocate_public_key_object(key_size: usize) -> Result<Self> {
        if key_size % 256 != 0 || key_size < 256 || key_size > 4096 {
            bail!("wrong key size");
        }
        let object = TransientObject::allocate(TransientObjectType::RsaPublicKey, key_size)
            .map_err(|e| anyhow!(e))?;
        Ok(Self {
            key_size,
            key_object: object,
            _key_object_type: TransientObjectType::RsaPublicKey,
        })
    }
    pub fn generate_key(&mut self) -> Result<()> {
        self.key_object
            .generate_key(self.key_size, &[])
            .map_err(|e| anyhow!(e))?;
        Ok(())
    }
    // get the value of an attribute
    pub fn ref_attribute(&self, attribute_id: AttributeId) -> Result<Vec<u8>> {
        let mut attribute = vec![0u8; self.key_size / 8];
        let output_size = self
            .key_object
            .ref_attribute(attribute_id, &mut attribute)
            .map_err(|e| anyhow!(e))?;
        println!(
            "RsaKeyObject::ref_attribute(): output_size: {}",
            output_size
        );
        Ok(attribute[..output_size].to_vec())
    }
    // set the value of attributes
    pub fn populate(&mut self, attrs: &[Attribute]) -> Result<()> {
        self.key_object.populate(attrs).map_err(|e| anyhow!(e))
    }
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key_info = self.key_object.info().map_err(|e| anyhow!(e))?;
        let size_in_bytes = key_info.object_size() / 8;
        ensure!(plaintext.len() <= size_in_bytes, "plaintext too long");

        match Asymmetric::allocate(
            AlgorithmId::RsaesPkcs1V15,
            OperationMode::Encrypt,
            key_info.object_size(),
        ) {
            Err(e) => bail!(e),
            Ok(cipher) => {
                cipher.set_key(&self.key_object).map_err(|e| anyhow!(e))?;
                match cipher.encrypt(&[], &plaintext) {
                    Err(e) => bail!(e),
                    Ok(cipher) => Ok(cipher.to_vec()),
                }
            }
        }
    }
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let key_info = self.key_object.info().map_err(|e| anyhow!(e))?;
        let size_in_bytes = key_info.object_size() / 8;
        ensure!(ciphertext.len() <= size_in_bytes, "ciphertext too long");

        match Asymmetric::allocate(
            AlgorithmId::RsaesPkcs1V15,
            OperationMode::Decrypt,
            key_info.object_size(),
        ) {
            Err(e) => bail!(e),
            Ok(cipher) => {
                cipher.set_key(&self.key_object)?;
                match cipher.decrypt(&[], &ciphertext) {
                    Err(e) => bail!(e),
                    Ok(plain) => Ok(plain.to_vec()),
                }
            }
        }
    }
}

impl TryFrom<RsaKeyObject> for RsaKeyPair {
    type Error = anyhow::Error;
    fn try_from(key_pair_object: RsaKeyObject) -> Result<Self> {
        let key_size = key_pair_object.key_size;
        let modulus = key_pair_object.ref_attribute(AttributeId::RsaModulus)?;
        let pub_exponent = key_pair_object.ref_attribute(AttributeId::RsaPublicExponent)?;
        let priv_exponent = key_pair_object.ref_attribute(AttributeId::RsaPrivateExponent)?;

        let public_key = RsaPublicKey::new(key_size, &modulus, &pub_exponent)?;
        let private_key = RsaPrivateKey::new(&priv_exponent)?;
        Ok(Self::new(key_size, public_key, private_key))
    }
}
impl TryInto<RsaKeyObject> for RsaKeyPair {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<RsaKeyObject> {
        let mut key_pair_object = RsaKeyObject::allocate_keypair_object(self.key_size())?;
        let attrs = vec![
            AttributeMemref::from_ref(AttributeId::RsaModulus, &self.public_key().modulus()).into(),
            AttributeMemref::from_ref(
                AttributeId::RsaPublicExponent,
                &self.public_key().exponent(),
            )
            .into(),
            AttributeMemref::from_ref(
                AttributeId::RsaPrivateExponent,
                &self.private_key().exponent(),
            )
            .into(),
        ];
        key_pair_object.populate(attrs.as_slice())?;
        Ok(key_pair_object)
    }
}
impl TryInto<RsaKeyObject> for RsaPublicKey {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<RsaKeyObject> {
        let mut key_pair_object = RsaKeyObject::allocate_public_key_object(self.key_size())?;
        let attrs = vec![
            AttributeMemref::from_ref(AttributeId::RsaModulus, &self.modulus()).into(),
            AttributeMemref::from_ref(AttributeId::RsaPublicExponent, &self.exponent()).into(),
        ];
        key_pair_object.populate(attrs.as_slice())?;
        Ok(key_pair_object)
    }
}
