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

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
ROOT_DIR="$SCRIPT_DIR/../.."
CRATES_DIR=(
    "optee-utee/optee-utee-sys"
    "optee-utee/macros"
    "optee-utee-build"
    "optee-utee"
    "optee-teec/optee-teec-sys"
    "optee-teec/macros"
    "optee-teec"
)

cd "$ROOT_DIR"
echo "Working directory set to: $(pwd)"

echo "=== Phase 1: Verify All Packages (Local Build Check) ==="
for DIR in "${CRATES_DIR[@]}"; do
    echo "[$DIR] Validating package..."
    if [ ! -d "$DIR" ]; then
        echo "Error: Directory $DIR not found!"
        exit 1
    fi
    (cd "$DIR" && cargo build)
done

echo -e "\nAll packages passed Phase 1 (local verification)."
echo "Entering Phase 2: Sequential Dry-run and Interactive Publish."

for DIR in "${CRATES_DIR[@]}"; do
    echo "------------------------------------------------"
    echo "Processing crate: $DIR"
    
    pushd "$DIR" > /dev/null

    echo "[$DIR] Step 1: Running cargo publish --dry-run..."
    cargo publish --dry-run

    echo -e "\n[$DIR] Dry-run successful."
    read -p "Do you want to formally PUBLISH [$DIR] to crates.io? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Publishing aborted by user at [$DIR]. Exiting."
        popd > /dev/null
        exit 1
    fi

    echo "[$DIR] Step 2: Publishing..."
    cargo publish

    popd > /dev/null

    echo "[$DIR] Done. Waiting 30s for crates.io index sync..."
    sleep 30
done

echo "------------------------------------------------"
echo "All crates have been published successfully!"
