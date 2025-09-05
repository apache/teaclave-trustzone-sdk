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

use attestation::report_verifier::{AttestationReport, ClientVerifier};
use attestation::sign::sign_csr_with_extension;
use basic_utils::println;
use crypto::EcdsaKeyPair;
use types::share::CkHash;

use crate::tls::handler_context::InterCertConfig;
use anyhow::Result;
use rustls::{ServerConfig, WantsVerifier};
use std::collections::HashSet;
use std::io::BufReader;
use std::sync::Arc;

/// Validation days of cert for TLS connection.
const CERT_VALID_DAYS: i64 = 365i64;
const CERT_VALID_SECONDS: i64 = CERT_VALID_DAYS * 24 * 60 * 60;

// this verifier is not used in the current version
fn client_verifier(_pubkey: &[u8]) -> bool {
    // println!("[+] ClientVerifier: pub_key: {:?}", pubkey);
    true
}

pub fn include_ca_cert() -> Vec<rustls::Certificate> {
    let bytes = include_bytes!("../../../pubkeys/ca.cert").to_vec();
    let cursor = std::io::Cursor::new(bytes);
    let mut reader = BufReader::new(cursor);
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|_| anyhow::anyhow!("pemfile::include_ca_cert failed"));
    certs
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

pub struct TlsServerConfigBuilder {
    pub config_builder: rustls::ConfigBuilder<ServerConfig, WantsVerifier>,
    pub cert_chain: Vec<rustls::Certificate>,
    pub end_key: rustls::PrivateKey,
}
impl TlsServerConfigBuilder {
    pub fn new(inter_cert_config: InterCertConfig, inter_keypair: EcdsaKeyPair) -> Result<Self> {
        let ca_cert = include_ca_cert();
        let attestation_report = AttestationReport {
            measurement: vec![0u8],
        };
        let report_payload = serde_json::to_vec(&attestation_report).unwrap();

        let end_kp = EcdsaKeyPair::new()?;
        let end_pub = end_kp.pub_key();
        let end_key = rustls::PrivateKey(end_kp.prv_key().to_vec());
        assert_eq!(
            end_pub[0], 0x4,
            "Missing uncompressed public key indicator."
        );

        let end_der = sign_csr_with_extension(
            &inter_keypair,
            end_pub,
            &inter_cert_config.issuer,  //"CK ECDSA level 2 intermediate",
            &inter_cert_config.subject, // "testserver.com",
            [&inter_cert_config.subject, "localhost"], // ["testserver.com", "localhost"],
            &report_payload,
            CERT_VALID_SECONDS,
        )?;
        let end_cert = rustls::Certificate(end_der.clone());
        println!("[+] ServerConfigBuilder::new(): sign end cert finished");
        let cert_chain = vec![end_cert, inter_cert_config.inter_cert, ca_cert[0].clone()];
        let config_builder = rustls::ServerConfig::builder().with_safe_defaults();
        println!("[+] ServerConfigBuilder::new(): setup server config finished");

        Ok(Self {
            config_builder,
            cert_chain,
            end_key,
        })
    }
    pub fn build_config_with_accepted_pubkey_hash(
        &self,
        accepted_pubkey_hash: HashSet<CkHash>,
    ) -> Result<Arc<ServerConfig>> {
        let hash: HashSet<Vec<u8>> = accepted_pubkey_hash.iter().map(|h| h.0.to_vec()).collect();
        let client_auth = Arc::new(ClientVerifier::new(hash, client_verifier));
        let config_builder = self.config_builder.clone();
        let mut config = config_builder
            .with_client_cert_verifier(client_auth)
            .with_single_cert(self.cert_chain.clone(), self.end_key.clone())
            .unwrap();
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});

        Ok(Arc::new(config))
    }
}
