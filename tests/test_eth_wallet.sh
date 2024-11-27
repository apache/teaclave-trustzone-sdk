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

# Include base script
source setup.sh

# Copy TA and host binary
cp ../projects/web3/eth_wallet/ta/target/$TARGET_TA/release/*.ta shared
cp ../projects/web3/eth_wallet/host/target/$TARGET_HOST/release/eth_wallet-rs shared

# Run script specific commands in QEMU
run_in_qemu "cp *.ta /lib/optee_armtz/\n"
run_in_qemu "./eth_wallet-rs test\n"
run_in_qemu "^C"

# Script specific checks
{
	grep -q "Tests passed" screenlog.0
} || {
        cat -v screenlog.0
        false
}

rm screenlog.0