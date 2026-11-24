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

# Serial 1: Normal World (NON_SECURE_WORLD_SERIAL, stdio by default).
# Serial 2: Secure World, logged to /tmp/serial.log.
cd "$1" || exit 1

./qemu-system-arm \
    -nodefaults \
    -nographic \
    -monitor none \
    -serial "${NON_SECURE_WORLD_SERIAL:-stdio}" -serial file:/tmp/serial.log \
    -smp 2 \
    -machine virt,secure=on \
    -cpu cortex-a15 \
    -d unimp -semihosting-config enable=on,target=native \
    -m 1057 \
    -bios bl1.bin \
    -object rng-random,filename=/dev/urandom,id=rng0 \
    -device virtio-rng-pci,rng=rng0,max-bytes=1024,period=1000 \
    -netdev user,id=vmnic,hostfwd=:127.0.0.1:54433-:4433,hostfwd=:127.0.0.1:54432-:22 \
    -device virtio-net-device,netdev=vmnic
