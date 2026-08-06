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

mkdir -p "$RUST_STD_DIR"
cd "$RUST_STD_DIR"

# Set up Rust and Cargo environment (RUSTUP_HOME and CARGO_HOME are set in bootstrap_env)
source "${CARGO_HOME}/env"

# initialize submodules: rust / libc with pinned versions for reproducible builds
RUST_TAG=1ed2df61a19042f231709eb05d032ae9e2cb2084 # nightly-2026-08-05
LIBC_TAG=0.2.189

echo "Cloning rust (tag: $RUST_TAG) and libc (tag: $LIBC_TAG)..."

# Clone official Rust source, then select the exact revision in RUST_TAG
git clone --filter=blob:none --no-checkout https://github.com/rust-lang/rust.git rust && \
    (cd rust && \
    git fetch --depth=1 origin "$RUST_TAG" && \
    git checkout --detach FETCH_HEAD && \
    git submodule update --init library/stdarch && \
    git submodule update --init library/backtrace)

# Clone official libc at specific tag
git clone --depth=1 --branch "$LIBC_TAG" https://github.com/rust-lang/libc.git

# Clone patches repository
git clone --depth=1 https://github.com/apache/teaclave-crates.git patches

# Apply patches
(cd rust && git apply ../patches/rust-1.99.0-1ed2df6/optee-0001-std-adaptation.patch)
(cd libc && git apply ../patches/libc-0.2.189-ef0906e/optee-0001-libc-adaptation.patch)

echo "rust-std initialized at $RUST_STD_DIR"
