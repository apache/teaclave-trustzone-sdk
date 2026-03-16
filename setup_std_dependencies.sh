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
# initialize submodules: rust / libc / patches
RUST_TAG=1.93.1        # commit 01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf
LIBC_TAG=0.2.182       # commit e879ee90b6cd8f79b352d4d4d1f8ca05f94f2f53

if [ -d rust/ ]
then
	rm -r rust/
fi

mkdir rust && cd rust

# Clone official Rust source at specific tag
git clone --depth=1 --branch $RUST_TAG https://github.com/rust-lang/rust.git && \
	(cd rust && \
	git submodule update --init library/stdarch && \
	git submodule update --init library/backtrace)

# Clone official libc at specific tag
git clone --depth=1 --branch $LIBC_TAG https://github.com/rust-lang/libc.git

# Clone patches repository
git clone --depth=1 https://github.com/apache/teaclave-crates.git patches

# Apply patches
(cd rust && git apply ../patches/rust-1.93.1-01f6ddf/optee-0001-std-adaptation.patch)
(cd libc && git apply ../patches/libc-0.2.182-e879ee9/optee-0001-libc-adaptation.patch)

echo "Rust and libc sources initialized and patched"
