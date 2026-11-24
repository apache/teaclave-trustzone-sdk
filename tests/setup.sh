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

OPTEE_TAG=optee-$(cat ../optee-version.txt)
IMG_SUFFIX="ubuntu-24.04-expand-ta-memory"

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
NW_SERIAL_LOG_PATH=/tmp/serial-nw.log

# A 32-bit client runs in the AArch32 guest; a 64-bit client runs in the AArch64 guest.
case "${TARGET_HOST:-}" in
    arm*)
        IMG="$(uname -m)-$OPTEE_TAG-qemu-arm32-$IMG_SUFFIX"
        QEMU_SCRIPT=./optee-qemu-arm32.sh
        CURRENT_SESSION_NAME=qemu_screen_arm32
        OTHER_SESSION_NAME=qemu_screen
        # AArch32 runs under TCG, which is noticeably slower than the AArch64 guest.
        RUN_IN_QEMU_TIMEOUT=20s
        BOOT_TIMEOUT=120s
        ;;
    aarch64*)
        IMG="$(uname -m)-$OPTEE_TAG-qemuv8-$IMG_SUFFIX"
        QEMU_SCRIPT=./optee-qemuv8.sh
        CURRENT_SESSION_NAME=qemu_screen
        OTHER_SESSION_NAME=qemu_screen_arm32
        RUN_IN_QEMU_TIMEOUT=10s
        BOOT_TIMEOUT=30s
        ;;
    *)
        echo "Unsupported TARGET_HOST: ${TARGET_HOST:-<unset>}" >&2
        echo "source environment first" >&2
        exit 1
        ;;
esac

# Function to download image
download_image() {
    curl "https://nightlies.apache.org/teaclave/teaclave-trustzone-sdk/${IMG}.tar.gz" | tar zxv
}

# Functions for running commands in QEMU
run_in_qemu() {
    run_in_qemu_with_timeout_secs "$1" "$RUN_IN_QEMU_TIMEOUT"
}

run_in_qemu_with_timeout_secs() {
    time timeout "$2" \
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
    if [ -f "$NW_SERIAL_LOG_PATH" ]; then
        cat -v "$NW_SERIAL_LOG_PATH"
    fi
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
    rm -f $SERIAL_LOG_PATH && rm -f $NW_SERIAL_LOG_PATH && rm -f $SCREEN_LOG_PATH
fi
# Start QEMU screen
if screen -list | grep -q "\.${CURRENT_SESSION_NAME}[[:space:]]"; then
    echo "Session '${CURRENT_SESSION_NAME}' is already running. Skipping start."
else
    echo "Starting new session: ${CURRENT_SESSION_NAME}"
    # Tests log Normal World to a file; the qemu scripts default to stdio.
    export NON_SECURE_WORLD_SERIAL=file:/tmp/serial-nw.log
    rm -f $SCREEN_LOG_PATH $SERIAL_LOG_PATH $NW_SERIAL_LOG_PATH
    screen -L -d -m -S $CURRENT_SESSION_NAME $QEMU_SCRIPT $IMG
fi

TEST_QEMU_SCRIPT_NAME=/tmp/teaclave-$CURRENT_SESSION_NAME.sh
cat <<EOF > "$TEST_QEMU_SCRIPT_NAME"
    until ssh -p $SSH_PORT $SSH_TARGET $SSH_OPTIONS "true" >/dev/null 2>&1; do
        printf "."
        sleep 1
    done
EOF
timeout $BOOT_TIMEOUT bash $TEST_QEMU_SCRIPT_NAME
echo "QEMU SSH Ready"
