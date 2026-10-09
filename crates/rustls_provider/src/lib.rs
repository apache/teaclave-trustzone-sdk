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

use optee_utee::{Random, Time};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::UnixTime;
use rustls::time_provider::TimeProvider;
use std::time::Duration;

/// Custom getrandom function using OP-TEE UTEE Random API
///
/// In getrandom 0.2 there is no built-in OP-TEE target, so we rely on the
/// `custom` feature to provide an OP-TEE RNG for the crypto provider.
/// Reference: https://docs.rs/getrandom/0.2.16/getrandom/macro.register_custom_getrandom.html
///
/// The shared `optee_getrandom` function is defined in this crate and
/// registered in the main.rs of tls client and server example.
pub fn optee_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    // Use OP-TEE's random number generator
    Random::generate(buf);
    Ok(())
}

/// CryptoProvider from rustls-rustcrypto, with the rng backend for OP-TEE in getrandom crate
pub fn optee_crypto_provider() -> CryptoProvider {
    rustls_rustcrypto::provider()
}

/// Custom TimeProvider implementation using OP-TEE UTEE API
#[derive(Debug)]
pub struct ReeTimeProvider;

impl TimeProvider for ReeTimeProvider {
    fn current_time(&self) -> Option<UnixTime> {
        // Get time from OP-TEE REE (Rich Execution Environment)
        // In normal operation, the value returned should correspond to the real time,
        // but it should not be considered as trusted, as it may be tampered by the user or the REE software.
        // reference: GPD_TEE_Internal_API_Specification
        let mut time = Time::new();
        time.ree_time();

        // Convert OP-TEE time to Unix timestamp
        // OP-TEE time seconds field represents seconds since some epoch
        // We need to treat it as Unix timestamp (seconds since Jan 1, 1970)
        let seconds = time.seconds as u64;
        let millis = time.millis as u64;

        // Create UnixTime from seconds and milliseconds, checking overflow
        seconds
            .checked_mul(1000)
            .and_then(|ms| ms.checked_add(millis))
            .map(|total| UnixTime::since_unix_epoch(Duration::from_millis(total)))
    }
}

pub fn optee_time_provider() -> ReeTimeProvider {
    ReeTimeProvider
}
