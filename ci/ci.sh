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

pushd ../tests

# Prioritize running specialized test suites first, as they have a higher
# probability of detecting failures early in the pipeline.
# Run std only tests
if [ "$STD" ]; then
    # move the EXPAND_TA_MEMORY tests together
    ./test_tls_client.sh
    ./test_tls_server.sh

    ./test_serde.sh
    ./test_message_passing_interface.sh
    ./test_eth_wallet.sh
    ./test_secure_db_abstraction.sh
else
    ./test_mnist_rs.sh
    ./test_build_with_optee_utee_sys.sh
fi

./test_hello_world.sh
./test_random.sh
./test_secure_storage.sh
./test_aes.sh
./test_hotp.sh
./test_acipher.sh
./test_big_int.sh
./test_diffie_hellman.sh
./test_digest.sh
./test_authentication.sh
./test_time.sh
./test_signature_verification.sh
./test_supp_plugin.sh
./test_error_handling.sh
./test_tcp_client.sh
./test_udp_socket.sh
./test_client_pool.sh
./test_inter_ta.sh
./test_property.sh


popd
