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

set -e

OPTEE_VERSION=$(cat ./optee-version.txt)
OPTEE_DIR=$1

# check arguments
if [ -z "$OPTEE_DIR" ]; then
    echo "Usage: $0 <optee_root_path>"
    exit 1
else
    export OPTEE_DIR=$OPTEE_DIR
    echo "OPTEE_DIR=$OPTEE_DIR"
fi

if [ ! -d $OPTEE_DIR ]; then
    mkdir -p $OPTEE_DIR
fi

# set toolchain
export CROSS_COMPILE32="arm-linux-gnueabihf-"
export CROSS_COMPILE64="aarch64-linux-gnu-"

# build optee_os and optee_client for qemu_v8
git clone https://github.com/OP-TEE/optee_os.git -b $OPTEE_VERSION $OPTEE_DIR/optee_os
(cd $OPTEE_DIR/optee_os && make PLATFORM=vexpress-qemu_armv8a)

git clone https://github.com/OP-TEE/optee_client.git -b $OPTEE_VERSION $OPTEE_DIR/optee_client
(cd $OPTEE_DIR/optee_client && make WITH_TEEACL=0 DESTDIR=$PWD/export_arm32 CROSS_COMPILE=$CROSS_COMPILE32)
(cd $OPTEE_DIR/optee_client && make clean && make WITH_TEEACL=0 DESTDIR=$PWD/export_arm64 CROSS_COMPILE=$CROSS_COMPILE64)
