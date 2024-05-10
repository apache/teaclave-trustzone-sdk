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
cp ../examples/supp_plugin-rs/ta/target/aarch64-unknown-linux-gnu/release/*.ta shared
cp ../examples/supp_plugin-rs/host/target/aarch64-unknown-linux-gnu/release/supp_plugin-rs shared
cp ../examples/supp_plugin-rs/plugin/target/aarch64-unknown-linux-gnu/release/*.plugin.so shared

# Run script specific commands in QEMU
run_in_qemu "cp *.ta /lib/optee_armtz/\n"
run_in_qemu "cp *.plugin.so /usr/lib/tee-supplicant/plugins/\n"
run_in_qemu "kill \$(pidof tee-supplicant)\n"
run_in_qemu "/usr/sbin/tee-supplicant &\n\n"
run_in_qemu "./supp_plugin-rs\n"
run_in_qemu "^C"

# Script specific checks
{
    grep -q "send value" screenlog.0 &&
    grep -q "invoke" screenlog.0 &&
    grep -q "receive value" screenlog.0 &&
    grep -q "invoke commmand finished" screenlog.0 &&
    grep -q "Success" screenlog.0
} || {
    cat -v screenlog.0
    cat -v /tmp/serial.log
    false
}

rm screenlog.0