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
copy_ta_to_qemu ../examples/ta/target/$TARGET_TA/release/*.ta
copy_ca_to_qemu ../examples/ca/target/$TARGET_HOST/release/tls_server-rs

# Run script specific commands in QEMU
run_in_qemu "nohup tls_server-rs > /tmp/tls_server.log 2>&1 &"

# Wait until the server listens on 4433. AArch32 TCG can be slow.
run_in_qemu_with_timeout_secs '
    while ! netstat -lnt | grep -q ":4433 "; do
        pidof tls_server-rs >/dev/null || exit 1
        sleep 1
    done
' 60 || {
    echo " [TIMEOUT] Server is not listening on 4433."
    run_in_qemu "cat /tmp/tls_server.log" || true
    print_detail_and_exit
}
# Outside the QEMU: connect the server using openssl, accept the self-signed CA cert
# || true because we want to continue testing even if the connection fails, and check the log later
OPENSSL_OUTPUT=$(echo "Q" | timeout 10s openssl s_client -connect 127.0.0.1:54433 -CAfile ../examples/ta/tls_server-rs/test-ca/ecdsa/ca.cert -debug 2>&1) || true
run_in_qemu "kill \$(pidof tls_server-rs)" || print_detail_and_exit

# Script specific checks
{
	grep -q "Verification: OK" <<< "$OPENSSL_OUTPUT" &&
	grep -v "SSL handshake has read 0 bytes " <<< "$OPENSSL_OUTPUT" # prevent openssl goes to http protocol
} || print_detail_and_exit
