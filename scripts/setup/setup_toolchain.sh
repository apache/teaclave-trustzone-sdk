#!/bin/bash

# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

set -xe

export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Ensure rustup is not already installed (we want fresh installation)
if command -v rustup &>/dev/null ; then
    echo "Error: rustup is already installed. This script requires a fresh installation." >&2
    exit 1
fi

# Clean installation of rustup with custom locations
echo "Installing rustup to ${RUSTUP_HOME} and cargo to ${CARGO_HOME}..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none

# Source the cargo environment from the new location to make rustup available for toolchain install
source ${CARGO_HOME}/env

# install the Rust toolchain set in rust-toolchain.toml
rustup toolchain install

##########################################
# install toolchain
if [[ "$(uname -m)" == "aarch64" ]]; then
    apt update && apt -y install gcc gcc-arm-linux-gnueabihf
else
    apt update && apt -y install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf
fi
