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
use anyhow::{bail, Result};
use std::convert::TryFrom;
use std::vec;

pub struct RsaKeyObject {
    key_size: usize,      // The number of bits in the modulus. can be 256, ..., 4096
    key_object: Vec<u8>,  // mocked for local testing
    key_object_type: u32, // mocked for local testing
}

impl RsaKeyObject {
    pub fn new(key_size: usize, key_object: Vec<u8>, key_object_type: u32) -> Result<Self> {
        Ok(Self {
            key_size,
            key_object,
            key_object_type,
        })
    }

    pub fn allocate_keypair_object(key_size: usize) -> Result<Self> {
        if key_size % 256 != 0 || key_size < 256 || key_size > 4096 {
            bail!("wrong key size, should be multiple of 256, between 256 and 4096");
        }
        Ok(Self {
            key_size,
            key_object: vec![],
            key_object_type: 0,
        })
    }

    pub fn generate_key(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // unimplemented, reserved for local compilation. We use optee backend for TAs.
        unimplemented!()
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // unimplemented, reserved for local compilation. We use optee backend for TAs.
        unimplemented!()
    }
}

impl TryFrom<RsaKeyObject> for RsaKeyPair {
    type Error = anyhow::Error;

    fn try_from(rsa_key_object: RsaKeyObject) -> Result<Self> {
        let pubkey = RsaPublicKey::new(rsa_key_object.key_size, &[], &[])?;
        let privkey = RsaPrivateKey::new(&[])?;
        Ok(RsaKeyPair::new(rsa_key_object.key_size, pubkey, privkey))
    }
}

impl TryInto<RsaKeyObject> for RsaPublicKey {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<RsaKeyObject> {
        // unimplemented, reserved for local compilation. We use optee backend for TAs.
        unimplemented!()
    }
}

impl TryInto<RsaKeyObject> for RsaKeyPair {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<RsaKeyObject> {
        // unimplemented, reserved for local compilation. We use optee backend for TAs.
        unimplemented!()
    }
}
