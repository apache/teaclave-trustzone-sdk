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

NEED_EXPANDED_MEM=true
# Include base script
source setup.sh

# Copy TA and host binary
copy_ta_to_qemu ../examples/tls_server-rs/ta/target/$TARGET_TA/release/*.ta
copy_ca_to_qemu ../examples/tls_server-rs/host/target/$TARGET_HOST/release/tls_server-rs

# Run script specific commands in QEMU
run_in_qemu "nohup tls_server-rs > /tmp/tls_server.log 2>&1 &"
(sleep 5 && run_in_qemu "cat /tmp/tls_server.log" | grep -q "listening") || (echo " [TIMEOUT] Server failed to start." && print_detail_and_exit)
# Outside the QEMU: connect the server using openssl, accept the self-signed CA cert
# || true because we want to continue testing even if the connection fails, and check the log later
OPENSSL_OUTPUT=$(echo "Q" | openssl s_client -connect 127.0.0.1:54433 -CAfile ../examples/tls_server-rs/ta/test-ca/ecdsa/ca.cert -debug 2>&1) || true
run_in_qemu "kill \$(pidof tls_server-rs)" || print_detail_and_exit

# Script specific checks
{
	grep -q "Verification: OK" <<< "$OPENSSL_OUTPUT" &&
	grep -v "SSL handshake has read 0 bytes " <<< "$OPENSSL_OUTPUT" # prevent openssl goes to http protocol
} || print_detail_and_exit
