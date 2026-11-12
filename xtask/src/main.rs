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

//! Repository maintenance tasks.
//!
//! ```text
//! cargo run --manifest-path xtask/Cargo.toml -- generate-bindings all
//! cargo run --manifest-path xtask/Cargo.toml -- generate-bindings utee
//! cargo run --manifest-path xtask/Cargo.toml -- generate-bindings teec
//! ```

mod generate_bindings;

use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "Repository maintenance tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Regenerate the committed bindings of the `*-sys` crates.
    GenerateBindings(generate_bindings::Args),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::GenerateBindings(args) => generate_bindings::run(args),
    }
}
