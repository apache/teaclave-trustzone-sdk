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

use anyhow::{bail, Result};
use clap::ValueEnum;
use std::env;
use std::path::PathBuf;
use std::process::Output;

/// Target architecture for building
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Arch {
    /// ARM 64-bit architecture
    Aarch64,
    /// ARM 32-bit architecture
    Arm,
}

/// Helper function to print command output and return error
pub fn print_output_and_bail(cmd_name: &str, output: &Output) -> Result<()> {
    eprintln!(
        "{} stdout: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stdout)
    );
    eprintln!(
        "{} stderr: {}",
        cmd_name,
        String::from_utf8_lossy(&output.stderr)
    );
    bail!(
        "{} failed with exit code: {:?}",
        cmd_name,
        output.status.code()
    )
}

/// Helper function to derive target and cross-compile prefix from arch
pub fn get_target_and_cross_compile(arch: Arch) -> (String, String) {
    match arch {
        Arch::Arm => (
            "arm-unknown-linux-gnueabihf".to_string(),
            "arm-linux-gnueabihf-".to_string(),
        ),
        Arch::Aarch64 => (
            "aarch64-unknown-linux-gnu".to_string(),
            "aarch64-linux-gnu-".to_string(),
        ),
    }
}

/// RAII guard to ensure we return to the original directory
pub struct ChangeDirectoryGuard {
    original: PathBuf,
}

impl ChangeDirectoryGuard {
    pub fn new(new_dir: &PathBuf) -> Result<Self> {
        let original = env::current_dir()?;
        env::set_current_dir(new_dir)?;
        Ok(Self { original })
    }
}

impl Drop for ChangeDirectoryGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}
