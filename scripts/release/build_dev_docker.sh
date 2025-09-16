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

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# Switch to root path
cd "$SCRIPT_DIR/../.."

# Paths relative to the script
DOCKERFILE="Dockerfile.dev"
OPTEE_VERSION_FILE="optee-version.txt"

# Read OP-TEE version
OPTEE_VER=$(cat "$OPTEE_VERSION_FILE")

echo "Building Docker images for OP-TEE version: $OPTEE_VER"

# Build no-std Docker image
docker build \
  -f "$DOCKERFILE" \
  --build-arg OPTEE_VERSION="$OPTEE_VER" \
  --target no-std-build-env \
  -t teaclave/teaclave-trustzone-emulator-nostd-optee-${OPTEE_VER}-expand-memory:latest \
  .

# Build std Docker image
docker build \
  -f "$DOCKERFILE" \
  --build-arg OPTEE_VERSION="$OPTEE_VER" \
  -t teaclave/teaclave-trustzone-emulator-std-optee-${OPTEE_VER}-expand-memory:latest \
  .

echo "Docker images built successfully!"

