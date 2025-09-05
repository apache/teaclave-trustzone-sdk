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
use std::convert::TryFrom;
use std::io::Cursor;
use std::io::{Read, Write};
use std::sync::Arc;
use std::vec;

use crate::tls::handler_context::TlsContext;
use attestation::verify::extract_pub_key_from_cert;

pub struct TlsSession {
    connection: rustls::ServerConnection,
    id: u32,
    client_pubkey_cache: Vec<u8>,
}
impl TlsSession {
    pub fn new(id: u32, server_config: Arc<rustls::ServerConfig>) -> Result<Self> {
        let connection = rustls::ServerConnection::new(server_config)?;
        println!("[+] TlsSession: new() id: {}", id);

        Ok(TlsSession {
            connection,
            id,
            client_pubkey_cache: Vec::new(),
        })
    }
    pub fn connection(&mut self) -> &mut rustls::ServerConnection {
        &mut self.connection
    }
    pub fn client_pubkey_cache(&mut self) -> Result<Vec<u8>> {
        if self.client_pubkey_cache.is_empty() {
            self.save_client_pubkey()?;
        }
        Ok(self.client_pubkey_cache.clone())
    }
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn read_request(&mut self, buf: &[u8]) -> Result<Vec<u8>> {
        let mut rd = Cursor::new(buf);
        let tls_connection = self.connection();
        let _rc = tls_connection.read_tls(&mut rd)?;
        let io_state = tls_connection.process_new_packets()?;

        // Read and process all available plaintext.
        let bytes_to_read = io_state.plaintext_bytes_to_read();
        if bytes_to_read > 0 {
            println!(
                "[+] read_request: plaintext_bytes_to_read: {:?}",
                bytes_to_read
            );
            let mut outbuf = vec![0u8; bytes_to_read];
            tls_connection.reader().read_exact(&mut outbuf)?;
            // save client pubkey
            self.save_client_pubkey()?;
            return Ok(outbuf);
        }
        Ok(Vec::new())
    }
    pub fn write_response(&mut self, buf: &[u8]) -> Result<()> {
        let tls_connection = self.connection();
        tls_connection.writer().write_all(buf)?;
        Ok(())
    }
    pub fn return_response_to_client(&mut self, buf: &mut [u8]) -> Result<usize> {
        let tls_connection = self.connection();
        let mut wr = Cursor::new(buf);
        let mut rc = 0;
        while tls_connection.wants_write() {
            rc += tls_connection.write_tls(&mut wr)?;
        }
        println!("[+] return_response_to_client: rc: {:?}", rc);
        Ok(rc)
    }

    fn save_client_pubkey(&mut self) -> Result<()> {
        let cert = self
            .connection
            .peer_certificates()
            .ok_or_else(|| anyhow!("no peer cert"))?[0]
            .clone();
        let client_pubkey = extract_pub_key_from_cert(&cert)?;
        println!("[+] TlsSession: save_client_pubkey as: {:?}", client_pubkey);
        self.client_pubkey_cache = client_pubkey;
        Ok(())
    }
}
impl TryFrom<&mut TlsSession> for TlsContext {
    type Error = anyhow::Error;
    fn try_from(tls_session: &mut TlsSession) -> Result<Self> {
        let client_pubkey = tls_session.client_pubkey_cache()?;
        Ok(TlsContext::new(client_pubkey))
    }
}
