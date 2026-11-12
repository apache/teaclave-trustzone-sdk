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

//! The `generate-bindings` command: regenerate the committed bindings of the
//! `*-sys` crates.
//!
//! One subcommand per system crate: `utee` covers `optee-utee-sys` (TA dev kit
//! headers), `teec` covers `optee-teec-sys` (client headers), and `all` runs
//! both.

mod teec;
mod utee;

use std::error::Error;

/// The target used for generation. The generated types do not depend on the
/// pointer width (C `size_t` becomes `usize`, `long` becomes `c_long`, pointers
/// stay pointers), so one run is valid for both 32-bit and 64-bit targets.
const TARGET: &str = "aarch64-unknown-linux-gnu";

/// Apache license header prepended to every generated file.
const LICENSE_HEADER: &str = include_str!("../../../license_header.txt");

#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(subcommand)]
    target: Target,
}

#[derive(clap::Subcommand)]
enum Target {
    /// Regenerate both crates.
    All {
        #[command(flatten)]
        teec: teec::TeecArgs,
        #[command(flatten)]
        utee: utee::UteeArgs,
    },
    /// Regenerate `crates/optee-teec-sys/src/generated.rs` from the client headers.
    Teec(teec::TeecArgs),
    /// Regenerate `crates/optee-utee-sys/src/generated.rs` from the TA dev kit.
    Utee(utee::UteeArgs),
}

pub(crate) fn run(args: Args) -> Result<(), Box<dyn Error>> {
    match args.target {
        Target::All { teec, utee } => {
            teec::run(teec)?;
            utee::run(utee)
        }
        Target::Teec(args) => teec::run(args),
        Target::Utee(args) => utee::run(args),
    }
}
