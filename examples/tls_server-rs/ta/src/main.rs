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

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{ErrorKind, Parameters, Result};
use proto::Command;

use anyhow::Context;
use lazy_static::lazy_static;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::sync::{Arc, Mutex, RwLock};

// Register the custom getrandom implementation.
//
// In getrandom 0.2 there is no built-in OP-TEE target, so we rely on the
// `custom` feature to provide an OP-TEE RNG.
// Reference: https://docs.rs/getrandom/0.2.16/getrandom/macro.register_custom_getrandom.html
//
// For this example, the shared `optee_getrandom` function is defined in the
// `rustls_provider` crate and registered here.
getrandom::register_custom_getrandom!(rustls_provider::optee_getrandom);

lazy_static! {
    static ref TLS_SESSIONS: RwLock<HashMap<u32, Mutex<rustls::ServerConnection>>> =
        RwLock::new(HashMap::new());
}

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
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    let session_id = unsafe { params.0.as_value()?.a() };
    trace_println!("[+] session id: {}", session_id);
    match Command::from(cmd_id) {
        Command::NewTlsSession => {
            trace_println!("[+] new_tls_session");
            new_tls_session(session_id).map_err(|e| {
                trace_println!("[-] Failed to create TLS session: {:?}", e);
                ErrorKind::Generic.into()
            })
        }
        Command::DoTlsRead => {
            let mut p1 = unsafe { params.1.as_memref()? };
            let buffer = p1.buffer();
            trace_println!("[+] do_tls_read");
            do_tls_read(session_id, buffer).map_err(|e| {
                trace_println!("[-] Failed to read TLS data: {:?}", e);
                ErrorKind::Generic.into()
            })
        }
        Command::DoTlsWrite => {
            trace_println!("[+] do_tls_write");
            let mut p1 = unsafe { params.1.as_memref()? };
            let mut p2 = unsafe { params.2.as_value()? };
            let buffer = p1.buffer();
            do_tls_write(session_id, buffer)
                .map(|n| {
                    p2.set_a(n as u32);
                })
                .map_err(|e| {
                    trace_println!("[-] Failed to write TLS data: {:?}", e);
                    ErrorKind::Generic.into()
                })
        }
        Command::CloseTlsSession => {
            trace_println!("[+] close_tls_session");
            close_tls_session(session_id).map_err(|e| {
                trace_println!("[-] Failed to close TLS session: {:?}", e);
                ErrorKind::Generic.into()
            })
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}

pub fn new_tls_session(session_id: u32) -> anyhow::Result<()> {
    let tls_config = make_config().context("Failed to create TLS config")?;
    let tls_session =
        rustls::ServerConnection::new(tls_config).context("Failed to create TLS connection")?;

    TLS_SESSIONS
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on TLS sessions"))?
        .insert(session_id, Mutex::new(tls_session));

    trace_println!("[+] TLS session {} created successfully", session_id);
    Ok(())
}

pub fn close_tls_session(session_id: u32) -> anyhow::Result<()> {
    let mut sessions = TLS_SESSIONS.write().map_err(|_| {
        anyhow::anyhow!(
            "Failed to acquire write lock to close TLS session {}",
            session_id
        )
    })?;

    if sessions.remove(&session_id).is_some() {
        trace_println!("[+] TLS session {} closed", session_id);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "TLS session {} not found for closing",
            session_id
        ))
    }
}

pub fn do_tls_read(session_id: u32, buf: &[u8]) -> anyhow::Result<()> {
    let mut rd = Cursor::new(buf);
    let ts_guard = TLS_SESSIONS
        .read()
        .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on TLS sessions"))?;

    let session = ts_guard
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("TLS session {} not found", session_id))?;

    let mut tls_session = session
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire lock on TLS session {}", session_id))?;

    tls_session
        .read_tls(&mut rd)
        .context("Failed to read TLS data")?;

    tls_session
        .process_new_packets()
        .context("Failed to process TLS packets")?;

    // Read and process all available plaintext.
    let mut buf = Vec::new();
    let _rc = tls_session.reader().read_to_end(&mut buf);
    if !buf.is_empty() {
        tls_session
            .writer()
            .write_all(&buf)
            .context("Failed to write response data")?;
    }

    Ok(())
}

pub fn do_tls_write(session_id: u32, buf: &mut [u8]) -> anyhow::Result<usize> {
    let ts_guard = TLS_SESSIONS
        .read()
        .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on TLS sessions"))?;

    let session = ts_guard
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("TLS session {} not found", session_id))?;

    let mut tls_session = session
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire lock on TLS session {}", session_id))?;

    let mut wr = Cursor::new(buf);
    let mut rc = 0;
    while tls_session.wants_write() {
        rc += tls_session
            .write_tls(&mut wr)
            .context("Failed to write TLS data")?;
    }

    Ok(rc)
}

fn make_config() -> anyhow::Result<Arc<rustls::ServerConfig>> {
    trace_println!("[+] Creating crypto provider");
    let crypto_provider = Arc::new(rustls_provider::optee_crypto_provider());

    trace_println!("[+] Creating time provider");
    let time_provider = Arc::new(rustls_provider::optee_time_provider());

    let certs = load_certs().context("Failed to load certificates")?;
    trace_println!("[+] Loaded {} certificates", certs.len());

    let private_key = load_private_key().context("Failed to load private key")?;
    trace_println!("[+] Private key loaded successfully");

    let config = rustls::ServerConfig::builder_with_details(crypto_provider, time_provider)
        .with_safe_default_protocol_versions()
        .context("Inconsistent cipher-suite/versions selected")?
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .context("Failed to create server config with certificate")?;

    Ok(Arc::new(config))
}

fn load_certs() -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let pem_data = include_bytes!("../test-ca/ecdsa/end.fullchain");
    let cursor = std::io::Cursor::new(pem_data);
    CertificateDer::pem_reader_iter(cursor)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to parse certificate PEM data")
}

fn load_private_key() -> anyhow::Result<PrivateKeyDer<'static>> {
    let pem_data = include_bytes!("../test-ca/ecdsa/end.key");
    let cursor = std::io::Cursor::new(pem_data);
    PrivateKeyDer::from_pem_reader(cursor).context("Failed to parse private key PEM data")
}

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
