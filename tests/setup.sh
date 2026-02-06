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

# Default value for NEED_EXPANDED_MEM
: ${NEED_EXPANDED_MEM:=false}
OPTEE_TAG=optee-$(cat ../optee-version.txt)

# Define IMG_VERSION
IMG_VERSION="$(uname -m)-$OPTEE_TAG-qemuv8-ubuntu-24.04"

IMG="$IMG_VERSION"
NORMAL_SESSION_NAME="qemu_screen"
EXPAND_MEMORY_SESSION_NAME="qemu_screen_expand_ta_memory"

CURRENT_SESSION_NAME=$NORMAL_SESSION_NAME
OTHER_SESSION_NAME=$EXPAND_MEMORY_SESSION_NAME
# Change Options based on NEED_EXPANDED_MEM
if [ "$NEED_EXPANDED_MEM" = true ]; then
    IMG="${IMG_VERSION}-expand-ta-memory"
    CURRENT_SESSION_NAME=$EXPAND_MEMORY_SESSION_NAME
    OTHER_SESSION_NAME=$NORMAL_SESSION_NAME
fi

SSH_PORT=54432
# StrictHostKeyChecking=no: Bypasses the interactive prompt to confirm the
#   host's authenticity.
# UserKnownHostsFile=/dev/null: Prevents saving host keys to disk; this avoids
#   "Host key verification failed" errors when the QEMU instance restarts with
#   a new identity.
SSH_OPTIONS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes"
SSH_TARGET="root@127.0.0.1"

SCREEN_LOG_PATH=screenlog.0
SERIAL_LOG_PATH=/tmp/serial.log

# Function to download image
download_image() {
    curl "https://nightlies.apache.org/teaclave/teaclave-trustzone-sdk/${IMG}.tar.gz" | tar zxv
}

# Functions for running commands in QEMU
run_in_qemu() {
    run_in_qemu_with_timeout_secs "$1" 10s
}

run_in_qemu_with_timeout_secs() {
    timeout "$2" \
        ssh $SSH_TARGET -p $SSH_PORT $SSH_OPTIONS "$1"
}

copy_to_qemu() {
    local dest_path=$1
    shift

    timeout 60s \
        scp -P $SSH_PORT $SSH_OPTIONS $@ $SSH_TARGET:"$dest_path"
}

copy_ta_to_qemu() {
    copy_to_qemu "/lib/optee_armtz/" $@
    run_in_qemu "chmod 0444 /lib/optee_armtz/*.ta"
}

copy_ca_to_qemu() {
    copy_to_qemu "/usr/bin/" $@
}

copy_plugin_to_qemu() {
    copy_to_qemu "/usr/lib/tee-supplicant/plugins/" $@
    run_in_qemu "chmod 0666 /usr/lib/tee-supplicant/plugins/*.so"
}

# Functions for handling failure
print_detail_and_exit() {
    cat -v $SCREEN_LOG_PATH
    cat -v $SERIAL_LOG_PATH
    exit 1
}

# Check if the image file exists locally
if [ ! -d "${IMG}" ]; then
    echo "Image file '${IMG}' not found locally. Downloading from network."
    download_image
else
    echo "Image file '${IMG}' found locally."
fi

mkdir -p shared
# Keeps the shared folder for ease of manual developer verification.
# "mkdir -p shared && mount -t 9p -o trans=virtio host shared"

# Terminate existing QEMU screen sessions to prevent conflicts.
if screen -list | grep -q "\.${OTHER_SESSION_NAME}[[:space:]]"; then
    echo "Other Session '${OTHER_SESSION_NAME}' is running, terminate it to prevent conflicts"
    screen -S $OTHER_SESSION_NAME -X quit
    rm -f $SERIAL_LOG_PATH && rm -f $SCREEN_LOG_PATH
fi
# Start QEMU screen
if screen -list | grep -q "\.${CURRENT_SESSION_NAME}[[:space:]]"; then
    echo "Session '${CURRENT_SESSION_NAME}' is already running. Skipping start."
else
    echo "Starting new session: ${CURRENT_SESSION_NAME}"
    screen -L -d -m -S $CURRENT_SESSION_NAME ./optee-qemuv8.sh $IMG
fi

TEST_QEMU_SCRIPT_NAME=/tmp/teaclave-$CURRENT_SESSION_NAME.sh
cat <<EOF > "$TEST_QEMU_SCRIPT_NAME"
    until ssh -p $SSH_PORT $SSH_TARGET $SSH_OPTIONS "true" >/dev/null 2>&1; do
        printf "."
        sleep 1
    done
EOF
timeout 30s bash $TEST_QEMU_SCRIPT_NAME
echo "QEMU SSH Ready"
