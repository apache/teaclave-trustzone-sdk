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

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InitBoardOpt {}

#[derive(Debug, StructOpt)]
pub struct BackupWalletOpt {
    /// backup devices' public keys signed by authority
    #[structopt(short, long, required = true)]
    pub signed_backup_list: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct RestoreWalletOpt {
    /// encrypted data
    #[structopt(short, long, required = true)]
    pub encrypted_data: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct ClearWalletStorageOpt {}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Initialize board, generate keypairs and return the public keys. If the board is already initialized, return the public keys.
    #[structopt(name = "init-board")]
    InitBoard(InitBoardOpt),

    /// Backup wallet data
    #[structopt(name = "backup-wallets")]
    BackupWallet(BackupWalletOpt),

    /// Restore wallet data
    #[structopt(name = "restore-wallets")]
    RestoreWallet(RestoreWalletOpt),

    /// Clear wallet storage
    #[structopt(name = "clear-wallet-storage")]
    ClearWalletStorage(ClearWalletStorageOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "device-cli-tool", about = "CK device management tool.")]
pub struct Opt {
    #[structopt(subcommand)]
    pub command: Command,
}
