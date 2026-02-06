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
copy_ta_to_qemu ../examples/supp_plugin-rs/ta/target/$TARGET_TA/release/*.ta
copy_ca_to_qemu ../examples/supp_plugin-rs/host/target/$TARGET_HOST/release/supp_plugin-rs
copy_plugin_to_qemu ../examples/supp_plugin-rs/plugin/target/$TARGET_HOST/release/*.plugin.so

# Run script specific commands in QEMU
run_in_qemu "kill \$(pidof tee-supplicant)"
run_in_qemu "nohup /usr/sbin/tee-supplicant > /tmp/tee_supplicant.log 2>&1 &"
OUTPUT=$(run_in_qemu "supp_plugin-rs && cat /tmp/tee_supplicant.log") || print_detail_and_exit

# Script specific checks
{
    grep -q "\*host\*: send value" <<< "$OUTPUT" &&
    grep -q "\*host\*: invoke" <<< "$OUTPUT" &&
    grep -q "Success" <<< "$OUTPUT"
    grep -q "\*plugin\*: invoke" <<< "$OUTPUT" &&
    grep -q "\*plugin\*: receive value" <<< "$OUTPUT" &&
    grep -q "\*plugin\*: send value" <<< "$OUTPUT"
} || print_detail_and_exit
