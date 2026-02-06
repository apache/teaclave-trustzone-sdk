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
copy_ta_to_qemu ../examples/client_pool-rs/ta/target/$TARGET_TA/release/*.ta
copy_ca_to_qemu ../examples/client_pool-rs/host/target/$TARGET_HOST/release/client_pool-rs

# Run script specific commands in QEMU
OUTPUT1=$(run_in_qemu "client_pool-rs thread -p 2 -c 2 -t 500 -e 2000") || print_detail_and_exit
OUTPUT2=$(run_in_qemu "client_pool-rs async -p 2 -c 2 -t 500 -e 2000") || print_detail_and_exit

# Script specific checks
{
    grep -q "r2d2: total tasks: 2, total finish: " <<< "$OUTPUT1" &&
    grep -q "mobc: total tasks: 2, total finish: " <<< "$OUTPUT2"
} || print_detail_and_exit
