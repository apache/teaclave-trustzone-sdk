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
copy_ta_to_qemu ../examples/mnist-rs/ta/target/$TARGET_TA/release/*.ta
copy_ca_to_qemu ../examples/mnist-rs/host/target/$TARGET_HOST/release/mnist-rs

# Run script specific commands in QEMU
# Do not export the model due to QEMU's memory limitations.
OUTPUT1=$(run_in_qemu_with_timeout_secs "mnist-rs train -n 1" 300) || print_detail_and_exit
# Copy samples files
copy_to_qemu "/tmp" ../examples/mnist-rs/host/samples/*
OUTPUT2=$(run_in_qemu "mnist-rs infer -m /tmp/model.bin -b /tmp/7.bin -i /tmp/7.png") || print_detail_and_exit

# Script specific checks
{
    grep -q "Train Success" <<< "$OUTPUT1" &&
    grep -q "Infer Success" <<< "$OUTPUT2"
} || print_detail_and_exit
