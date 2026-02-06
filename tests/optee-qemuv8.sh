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

cd $1 && ./qemu-system-aarch64 \
    -nodefaults \
    -nographic \
    -serial stdio -serial file:/tmp/serial.log \
    -smp 2 \
    -s -machine virt,secure=on,acpi=off,gic-version=3 \
    -cpu cortex-a57 \
    -d unimp -semihosting-config enable=on,target=native \
    -m 1057 \
    -bios bl1.bin \
    -initrd rootfs.cpio.gz \
    -append 'console=ttyAMA0,115200 keep_bootcon root=/dev/vda2' \
    -kernel Image \
    -fsdev local,id=fsdev0,path=$(pwd)/../shared,security_model=none \
    -device virtio-9p-device,fsdev=fsdev0,mount_tag=host \
    -netdev user,id=vmnic,hostfwd=:127.0.0.1:54433-:4433,hostfwd=:127.0.0.1:54432-:22 \
    -device virtio-net-device,netdev=vmnic
