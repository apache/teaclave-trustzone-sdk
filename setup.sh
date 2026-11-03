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

##########################################
# move to project root
cd "$(dirname "$0")"

##########################################
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# install rustup if needed
if command -v rustup &>/dev/null ; then
    # rustup early than 1.28 fails with `rustup toolchain install`
    # due to parameter mismatch. So self update first.
    rustup self update
else
    # Do not install the default stable toolchain. The pinned toolchain
    # is installed from rust-toolchain.toml below.
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    source "$HOME/.cargo/env"
fi

# install the Rust toolchain set in rust-toolchain.toml
rustup toolchain install

##########################################
# install toolchain
if [[ "$(uname -m)" == "aarch64" ]]; then
    apt update && apt -y install gcc gcc-arm-linux-gnueabihf
else
    apt update && apt -y install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf
fi
