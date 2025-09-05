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

# Path to ConfidentialKlave
CK_ROOT_PATH=$1
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TMP=$SCRIPT_DIR/tmp

# check if paths are set
if [ -z "$CK_ROOT_PATH" ]; then
  echo "Usage: install_certs.sh <CK-root-path>"
  exit 1
fi
if [ ! -d $TMP ]; then
  echo "$TMP does not exist. Please run generate_certs.sh first."
  exit 1 
fi

# install pubkeys into ConfidentialKlave
CK_PUBKEY_PATH=$CK_ROOT_PATH/pubkeys
if [ ! -d $CK_PUBKEY_PATH ]; then
  mkdir $CK_PUBKEY_PATH
fi
cp $TMP/ca.cert $TMP/ca.pub $TMP/system.pub $CK_PUBKEY_PATH

echo "Certs and pubkeys installed to $CK_PUBKEY_PATH"
