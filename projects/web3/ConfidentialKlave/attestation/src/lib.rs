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

#[macro_use]
pub mod cert;
pub mod date;
pub mod report_verifier;
pub mod sign;
pub mod utils;
pub mod verify;

#[cfg(test)]
mod tests {
    use crate::report_verifier::*;
    use crate::sign::{sign_csr_with_extension, sign_intermediate};
    use crate::utils::*;
    use crate::verify;
    use crate::verify::AttestationCertChain;
    use crypto::{EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P384_SHA384_ASN1_SIGNING};

    use anyhow::Result;
    use rustls::RootCertStore;
    use rustls::{ConnectionCommon, SideData};
    use std::collections::HashSet;
    use std::convert::TryInto;
    use std::io::{self, Read};
    use std::iter::FromIterator;
    use std::ops::{Deref, DerefMut};
    use std::sync::Arc;

    const TEST_CERT_VAILD_SECONDS: i64 = 60 * 60 * 24 * 365;

    fn setup_ra_client_config(
        report_payload: &[u8],
    ) -> Result<(rustls::ClientConfig, AttestationCertChain)> {
        // Note: for generate test-ca certs, please refer to the tls_server example in Teaclave SDK
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let inter_cert_path = "test-ca/ecdsa/inter.cert";
        let inter_key_path = "test-ca/ecdsa/inter.key";
        let client_key_path = "test-ca/ecdsa/client.key";

        // inter.key sign client cert
        let at_key = load_pem_key(inter_key_path)?;
        let ak_kp = EcdsaKeyPair::from_bytes(&at_key.0, &ECDSA_P256_SHA256_ASN1_SIGNING)?;

        let client_key = load_pem_key(client_key_path)?;
        let client_kp = EcdsaKeyPair::from_bytes(&client_key.0, &ECDSA_P256_SHA256_ASN1_SIGNING)?;
        let client_pub = client_kp.pub_key();
        assert_eq!(
            client_pub[0], 0x4,
            "Missing uncompressed public key indicator."
        );

        let client_der = sign_csr_with_extension(
            &ak_kp,
            client_pub,
            "CK ECDSA level 2 intermediate",
            "CK client",
            ["client", "localhost"],
            report_payload,
            TEST_CERT_VAILD_SECONDS,
        )?;
        let client_cert = rustls::Certificate(client_der);

        // setup ca cert
        let mut root_store = RootCertStore::empty();
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        root_store.add_parsable_certificates(&[ca_cert.0.clone()]);

        // setup key and client cert fullchain
        let inter_cert = load_pem_single_cert(inter_cert_path)?;
        let acc = AttestationCertChain {
            end: client_cert.clone().0,
            intermediate: inter_cert.clone().0,
            root: ca_cert.clone().0,
        };
        let cert_chain = vec![client_cert, inter_cert, ca_cert];

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_single_cert(cert_chain, client_key)
            .unwrap();

        Ok((config, acc))
    }

    fn setup_fixed_client_config() -> Result<rustls::ClientConfig> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let client_fullchain_path = "test-ca/ecdsa/client.fullchain";
        let client_key_path = "test-ca/ecdsa/client.key";

        // setup ca cert
        let mut root_store = RootCertStore::empty();
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        root_store.add_parsable_certificates(&[ca_cert.0.clone()]);

        // setup key and client cert fullchain
        let cert_chain = load_pem_cert(client_fullchain_path)?;
        let client_key = load_pem_key(client_key_path)?;

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_single_cert(cert_chain, client_key)
            .unwrap();

        Ok(config)
    }

    fn server_verifier(report: &AttestationReport) -> bool {
        println!("[+] server_verifier: attestation report: {:?}", report);
        true
    }

    fn setup_client_config_with_verifier() -> Result<rustls::ClientConfig> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let client_fullchain_path = "test-ca/ecdsa/client.fullchain";
        let client_key_path = "test-ca/ecdsa/client.key";

        let accepted_enclave_attrs = vec![EnclaveAttr {
            measurement: vec![0u8],
        }];
        let root_cert_bytes = include_bytes!("../test-ca/ecdsa/ca.der");
        let verifier = Arc::new(AttestationReportVerifier::new(
            accepted_enclave_attrs,
            root_cert_bytes,
            server_verifier,
        ));
        let mut root_store = RootCertStore::empty();
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        root_store.add_parsable_certificates(&[ca_cert.0.clone()]);
        let cert_chain = load_pem_cert(client_fullchain_path)?;
        let client_key = load_pem_key(client_key_path)?;

        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_single_cert(cert_chain, client_key)
            .unwrap();
        config.dangerous().set_certificate_verifier(verifier);

        Ok(config)
    }

    fn setup_client_config_with_dyn_inter_cert_verifier() -> Result<rustls::ClientConfig> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let client_fullchain_path = "test-ca/ecdsa/client.fullchain";
        let client_key_path = "test-ca/ecdsa/client.key";

        let accepted_enclave_attrs = vec![EnclaveAttr {
            measurement: vec![0u8],
        }];
        let root_cert_bytes = include_bytes!("../test-ca/ecdsa/ca.der");
        let verifier = Arc::new(AttestationReportVerifier::new(
            accepted_enclave_attrs,
            root_cert_bytes,
            server_verifier,
        ));
        let mut root_store = RootCertStore::empty();
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        root_store.add_parsable_certificates(&[ca_cert.0.clone()]);
        let cert_chain = load_pem_cert(client_fullchain_path)?;
        let client_key = load_pem_key(client_key_path)?;

        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_single_cert(cert_chain, client_key)
            .unwrap();
        config.dangerous().set_certificate_verifier(verifier);

        Ok(config)
    }

    fn setup_fixed_server_config() -> Result<rustls::ServerConfig> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let server_fullchain_path = "test-ca/ecdsa/end.fullchain";
        let server_key_path = "test-ca/ecdsa/end.key";

        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        let mut client_auth_roots = rustls::RootCertStore::empty();
        client_auth_roots.add(&ca_cert)?;
        let client_auth = rustls::server::AllowAnyAuthenticatedClient::new(client_auth_roots);
        let cert = load_pem_cert(server_fullchain_path)?;
        let key = load_pem_key(server_key_path)?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(client_auth))
            .with_single_cert(cert, key)
            .unwrap();

        Ok(config)
    }

    fn setup_ra_server_config(
        report_payload: &[u8],
    ) -> Result<(rustls::ServerConfig, AttestationCertChain)> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let inter_cert_path = "test-ca/ecdsa/inter.cert";
        let inter_key_path = "test-ca/ecdsa/inter.key";
        let end_key_path = "test-ca/ecdsa/end.key";

        // server verify client cert
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        let mut client_auth_roots = rustls::RootCertStore::empty();
        client_auth_roots.add(&ca_cert)?;
        let client_auth = rustls::server::AllowAnyAuthenticatedClient::new(client_auth_roots);

        // inter.key sign server end cert
        let at_key = load_pem_key(inter_key_path)?;
        let ak_kp = EcdsaKeyPair::from_bytes(&at_key.0, &ECDSA_P256_SHA256_ASN1_SIGNING)?;

        let end_key = load_pem_key(end_key_path)?;
        let end_kp = EcdsaKeyPair::from_bytes(&end_key.0, &ECDSA_P256_SHA256_ASN1_SIGNING)?;
        let end_pub = end_kp.pub_key();
        assert_eq!(
            end_pub[0], 0x4,
            "Missing uncompressed public key indicator."
        );

        let end_der = sign_csr_with_extension(
            &ak_kp,
            end_pub,
            "CK ECDSA level 2 intermediate",
            "testserver.com",
            ["testserver.com", "localhost"],
            report_payload,
            TEST_CERT_VAILD_SECONDS,
        )?;
        let end_cert = rustls::Certificate(end_der.clone());
        println!("setup server config: sign");

        // setup key and server cert fullchain
        let inter_cert = load_pem_single_cert(inter_cert_path)?;
        let acc = AttestationCertChain {
            end: end_cert.clone().0,
            intermediate: inter_cert.clone().0,
            root: ca_cert.clone().0,
        };

        let cert_chain = vec![end_cert, inter_cert, ca_cert];

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(client_auth))
            .with_single_cert(cert_chain, end_key)
            .unwrap();

        Ok((config, acc))
    }

    fn client_verifier(_cert: &[u8]) -> bool {
        println!("[+] client_verifier");
        true
    }

    fn setup_ra_dyn_inter_cert_server_config(
        report_payload: &[u8],
    ) -> Result<(rustls::ServerConfig, AttestationCertChain)> {
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let ca_key_path = "test-ca/ecdsa/ca.key";

        // load ca cert and key
        let ca_cert = load_pem_single_cert(ca_cert_path)?;
        let ca_key = load_pem_key(ca_key_path)?;
        let mut client_auth_roots = rustls::RootCertStore::empty();
        client_auth_roots.add(&ca_cert)?;
        let client_auth = rustls::server::AllowAnyAuthenticatedClient::new(client_auth_roots);

        // generate intermediate cert
        let ca_kp = EcdsaKeyPair::from_bytes(&ca_key.0, &ECDSA_P384_SHA384_ASN1_SIGNING)?;
        let inter_kp = EcdsaKeyPair::new()?;
        let inter_pub = inter_kp.pub_key();
        assert_eq!(
            inter_pub[0], 0x4,
            "Missing uncompressed public key indicator."
        );
        let inter_der = sign_intermediate(
            &ca_kp,
            inter_pub,
            "CK ECDSA CA",
            "CK ECDSA level 2 intermediate",
            TEST_CERT_VAILD_SECONDS,
        )?;
        let inter_cert = rustls::Certificate(inter_der.clone());
        println!("setup intermedate cert finished");

        // inter.key sign server end cert
        let end_kp = EcdsaKeyPair::new()?;
        let end_pub = end_kp.pub_key();
        assert_eq!(
            end_pub[0], 0x4,
            "Missing uncompressed public key indicator."
        );

        let end_der = sign_csr_with_extension(
            &inter_kp,
            end_pub,
            "CK ECDSA level 2 intermediate",
            "testserver.com",
            ["testserver.com", "localhost"],
            report_payload,
            TEST_CERT_VAILD_SECONDS,
        )?;
        println!("setup intermedate cert finished");
        let end_cert = rustls::Certificate(end_der.clone());

        // setup cert fullchain
        let acc = AttestationCertChain {
            end: end_cert.clone().0,
            intermediate: inter_cert.clone().0,
            root: ca_cert.clone().0,
        };

        let cert_chain = vec![end_cert, inter_cert, ca_cert];
        let end_key = rustls::PrivateKey(end_kp.prv_key().to_vec());
        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(client_auth))
            .with_single_cert(cert_chain, end_key)
            .unwrap();

        Ok((config, acc))
    }

    fn setup_server_config_with_verifier() -> Result<rustls::ServerConfig> {
        let server_fullchain_path = "test-ca/ecdsa/end.fullchain";
        let server_key_path = "test-ca/ecdsa/end.key";

        let accepted_pub_keys = HashSet::from_iter([vec![0u8; 64]]);
        let client_auth = ClientVerifier::new(accepted_pub_keys, client_verifier);

        let cert = load_pem_cert(server_fullchain_path)?;
        let key = load_pem_key(server_key_path)?;

        let config = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(client_auth))
            .with_single_cert(cert, key)
            .unwrap();

        Ok(config)
    }

    fn transfer<L, R, LS, RS>(left: &mut L, right: &mut R, expect_data: Option<usize>)
    where
        L: DerefMut + Deref<Target = ConnectionCommon<LS>>,
        R: DerefMut + Deref<Target = ConnectionCommon<RS>>,
        LS: SideData,
        RS: SideData,
    {
        let mut tls_buf = [0u8; 262144];
        let mut data_left = expect_data;
        let mut data_buf = [0u8; 8192];

        loop {
            let mut sz = 0;

            while left.wants_write() {
                let written = left.write_tls(&mut tls_buf[sz..].as_mut()).unwrap();
                if written == 0 {
                    break;
                }

                sz += written;
            }

            if sz == 0 {
                return;
            }

            let mut offs = 0;
            loop {
                match right.read_tls(&mut tls_buf[offs..sz].as_ref()) {
                    Ok(read) => {
                        right.process_new_packets().unwrap();
                        offs += read;
                    }
                    Err(err) => {
                        panic!("error on transfer {}..{}: {}", offs, sz, err);
                    }
                }

                if let Some(left) = &mut data_left {
                    loop {
                        let sz = match right.reader().read(&mut data_buf) {
                            Ok(sz) => sz,
                            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
                            Err(err) => panic!("failed to read data: {}", err),
                        };

                        *left -= sz;
                        if *left == 0 {
                            break;
                        }
                    }
                }
                if sz == offs {
                    break;
                }
            }
        }
    }

    pub fn do_handshake(
        client: &mut rustls::ClientConnection,
        server: &mut rustls::ServerConnection,
    ) -> bool {
        while server.is_handshaking() || client.is_handshaking() {
            transfer(client, server, None);
            server.process_new_packets().unwrap();
            transfer(server, client, None);
            client.process_new_packets().unwrap();
        }
        true
    }

    #[test]
    fn tls_ra_works() {
        let server_config = setup_fixed_server_config().unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();

        let report = AttestationReport::default();
        let payload = serde_json::to_vec(&report).unwrap();

        let (client_config, acc) = setup_ra_client_config(&payload).unwrap();
        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();

        do_handshake(&mut client, &mut server);

        /* Ideal solution: get certs in server handler */
        let certs = server.peer_certificates().unwrap();
        let report_recv = verify::extract_report_from_cert(&certs[0]).unwrap();
        assert_eq!(report, report_recv);

        /* Temporary solution: get AttestationCertChain in POST parameters */
        let ca_cert_path = "test-ca/ecdsa/ca.cert";
        let server_ca = load_pem_single_cert(ca_cert_path).unwrap();
        verify::verify_cert_chain(acc, &server_ca).unwrap();
        let _cert_info = verify::extract_info_from_cert(&certs[0]).unwrap();
        let report_recv = verify::extract_report_from_cert(&certs[0]).unwrap();
        assert_eq!(report, report_recv);
    }

    #[test]
    fn mutual_tls_works() {
        let client_config = setup_fixed_client_config().unwrap();
        let server_config = setup_fixed_server_config().unwrap();

        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
        do_handshake(&mut client, &mut server);
    }

    #[test]
    fn tls_server_ra_works() {
        let report = AttestationReport::default();
        let payload = serde_json::to_vec(&report).unwrap();
        let (server_config, _acc) = setup_ra_server_config(&payload).unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();

        let client_config = setup_fixed_client_config().unwrap();
        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        do_handshake(&mut client, &mut server);

        /* Ideal solution: get certs in server handler */
        let server_ca = load_pem_single_cert("test-ca/ecdsa/ca.cert").unwrap();
        let certs = client.peer_certificates().unwrap();
        let chain = AttestationCertChain {
            end: certs[0].0.clone(),
            intermediate: certs[1].0.clone(),
            root: certs[2].0.clone(),
        };

        verify::verify_cert_chain(chain, &server_ca).unwrap();
        let report_recv = verify::extract_report_from_cert(&certs[0]).unwrap();
        assert_eq!(report, report_recv);
    }

    // test client public key from server side
    #[test]
    fn test_client_verifier_works() {
        // client cert should be generated by yasna in order to extract the public key
        let report = AttestationReport::default();
        let payload = serde_json::to_vec(&report).unwrap();
        let (client_config, _acc) = setup_ra_client_config(&payload).unwrap();
        let server_config = setup_server_config_with_verifier().unwrap();

        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
        assert!(do_handshake(&mut client, &mut server));
    }

    // test server cert from client side
    #[test]
    fn test_server_verifier_works() {
        let client_config = setup_client_config_with_verifier().unwrap();
        let report = AttestationReport::default();
        let payload = serde_json::to_vec(&report).unwrap();
        let (server_config, _acc) = setup_ra_server_config(&payload).unwrap();

        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
        assert!(do_handshake(&mut client, &mut server));
    }

    // test dyn intermediate cert from client side
    #[test]
    fn test_dyn_inter_server_verifier_works() {
        let client_config = setup_client_config_with_dyn_inter_cert_verifier().unwrap();
        let report = AttestationReport::default();
        let payload = serde_json::to_vec(&report).unwrap();
        let (server_config, _acc) = setup_ra_dyn_inter_cert_server_config(&payload).unwrap();

        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
        assert!(do_handshake(&mut client, &mut server));
    }
}
