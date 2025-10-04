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

# Validate required environment variables
: "${RUST_STD_DIR:?RUST_STD_DIR must be set - directory where rust-std will be installed}"

# expected layout
#  $RUST_STD_DIR/rust
#  $RUST_STD_DIR/libc
#  $RUST_STD_DIR/$customized-target1.json
#  $RUST_STD_DIR/$customized-target2.json

mkdir -p $RUST_STD_DIR
cd $RUST_STD_DIR

# Set up Rust and Cargo environment (RUSTUP_HOME and CARGO_HOME are set in bootstrap_env)
source ${CARGO_HOME}/env

# install Xargo if not exist
which xargo || cargo install xargo

# initialize submodules: rust / libc with pinned versions for reproducible builds
RUST_BRANCH=optee-xargo
RUST_COMMIT=HEAD  # TODO: Pin to specific commit hash for reproducible builds
LIBC_BRANCH=optee  
LIBC_COMMIT=HEAD  # TODO: Pin to specific commit hash for reproducible builds

echo "Cloning rust (branch: $RUST_BRANCH) and libc (branch: $LIBC_BRANCH)..."

git clone --depth=1 -b $RUST_BRANCH https://github.com/DemesneGH/rust.git && \
       (cd rust && \
       git submodule update --init library/stdarch && \
       git submodule update --init library/backtrace)

git clone --depth=1 -b $LIBC_BRANCH https://github.com/DemesneGH/libc.git

echo "rust-std initialized at $RUST_STD_DIR"
