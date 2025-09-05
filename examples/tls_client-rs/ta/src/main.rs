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

#![no_main]

use anyhow::Context;
use optee_utee::net::TcpStream;
use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;
use rustls::RootCertStore;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::sync::Arc;

// Register the custom getrandom implementation.
//
// In getrandom 0.2 there is no built-in OP-TEE target, so we rely on the
// `custom` feature to provide an OP-TEE RNG.
// Reference: https://docs.rs/getrandom/0.2.16/getrandom/macro.register_custom_getrandom.html
//
// For this example, the shared `optee_getrandom` function is defined in the
// `rustls_provider` crate and registered here.
getrandom::register_custom_getrandom!(rustls_provider::optee_getrandom);

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA open session");
    Ok(())
}

#[ta_close_session]
fn close_session() {
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::Start => match tls_client() {
            Ok(_) => {
                trace_println!("[+] TLS client completed successfully");
                Ok(())
            }
            Err(e) => {
                trace_println!("[-] TLS client failed: {:?}", e);
                Err(ErrorKind::Generic.into())
            }
        },
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

// This code is based on the Rustls example:
// https://github.com/rustls/rustls/blob/v/0.23.12/examples/src/bin/simpleclient.rs
// with modifications by Teaclave to demonstrate Rustls usage in the TA.
// Licensed under the Apache License, Version 2.0.
fn tls_client() -> anyhow::Result<()> {
    // Create our custom providers
    let crypto_provider = Arc::new(rustls_provider::optee_crypto_provider());
    let time_provider = Arc::new(rustls_provider::optee_time_provider());

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };

    let config = rustls::ClientConfig::builder_with_details(crypto_provider, time_provider)
        .with_safe_default_protocol_versions()
        .context("Failed to create client config with safe default protocol versions")?
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = "www.rust-lang.org"
        .try_into()
        .context("Failed to parse server name")?;

    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name)
        .context("Failed to create client connection")?;

    let mut sock =
        TcpStream::connect("www.rust-lang.org", 443).context("Failed to connect to server")?;

    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    tls.write_all(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: www.rust-lang.org\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        )
        .as_bytes(),
    )
    .context("Failed to write HTTP request")?;

    let ciphersuite = tls
        .conn
        .negotiated_cipher_suite()
        .context("Failed to get negotiated cipher suite")?;
    trace_println!("Current ciphersuite: {:?}", ciphersuite.suite());

    let mut plaintext = Vec::new();
    tls.read_to_end(&mut plaintext)
        .context("Failed to read response")?;
    trace_println!("{}", String::from_utf8_lossy(&plaintext));

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
