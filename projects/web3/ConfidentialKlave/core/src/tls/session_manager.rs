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
use basic_utils::println;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use types::share::CkHash;

use crate::key_utils::load_signing_keypair;
use crate::secure_storage_client::SecureStorageClient;
use crate::tls::config_builder::TlsServerConfigBuilder;
use crate::tls::handler_context::{InterCertConfig, TlsContext};
use crate::tls::session::TlsSession;

pub struct TlsSessionManager {
    inner: HashMap<u32, TlsSession>,
    config_builder: Option<TlsServerConfigBuilder>,
}
impl TlsSessionManager {
    pub fn init() -> Self {
        Self {
            inner: HashMap::new(),
            config_builder: None,
        }
    }
    pub fn init_tls_server(&mut self, inter_cert_config: InterCertConfig) -> Result<()> {
        let db_client = SecureStorageClient::init();
        let inter_kp = load_signing_keypair(&db_client)?;
        let config_builder = TlsServerConfigBuilder::new(inter_cert_config, inter_kp)?;
        println!("[+] init_tls_server: server config builder initialized");
        self.config_builder = Some(config_builder);
        Ok(())
    }
    pub fn new_tls_session(
        &mut self,
        session_id: u32,
        accepted_pubkey_hash: HashSet<CkHash>,
    ) -> Result<()> {
        let server_config = self
            .config_builder
            .as_ref()
            .ok_or_else(|| anyhow!("server config not initialized"))?
            .build_config_with_accepted_pubkey_hash(accepted_pubkey_hash)?;
        println!(
            "[+] new_tls_session: server config constructed for session_id: {:?}",
            session_id
        );

        let tls_session = TlsSession::new(session_id, server_config)?;
        self.inner.insert(session_id, tls_session);
        Ok(())
    }
    pub fn do_tls_read(&mut self, session_id: u32, buf: &[u8]) -> Result<Vec<u8>> {
        let tls_session = self
            .inner
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session_id not found"))?;
        tls_session.read_request(buf)
    }
    pub fn write_response(&mut self, session_id: u32, response: &[u8]) -> Result<()> {
        let tls_session = self
            .inner
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session_id not found"))?;
        tls_session.write_response(response)
    }
    pub fn do_tls_write(&mut self, session_id: u32, buf: &mut [u8]) -> Result<usize> {
        let tls_session = self
            .inner
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session_id not found"))?;
        tls_session.return_response_to_client(buf)
    }
    pub fn close_tls_session(&mut self, session_id: u32) -> Result<()> {
        let _tls_session = self
            .inner
            .remove(&session_id)
            .ok_or_else(|| anyhow!("session_id not found"))?;
        Ok(())
    }
    pub fn construct_tls_context(&mut self, session_id: u32) -> Result<TlsContext> {
        let tls_session = self
            .inner
            .get_mut(&session_id)
            .ok_or_else(|| anyhow!("session_id not found"))?;
        tls_session
            .try_into()
            .map_err(|e| anyhow!("construct_tls_context failed: {:?}", e))
    }
}
