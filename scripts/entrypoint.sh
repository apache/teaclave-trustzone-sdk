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

# Docker entrypoint script for Teaclave TrustZone SDK development environment

set -e

# Unified switch_config function as a wrapper for switch_config in the toolchain
# The function reloads the environment in current shell after switch_config is executed
define_switch_config() {
    cat << 'EOF'
switch_config() {
    if ${TEACLAVE_TOOLCHAIN_BASE}/bin/switch_config "$@"; then
        case "$1" in
            --ta|--host)
                echo ""
                echo "Reloading environment in current shell..."
                source ${TEACLAVE_TOOLCHAIN_BASE}/environment
                echo "✅ Configuration applied to current shell!"
                ;;
        esac
    fi
}
export -f switch_config
EOF
}

# Set up bash profile with all necessary environment sources
setup_bash_profile() {
    echo "Setting up bash profile environment..."
    
    # Clear existing bashrc to avoid circular sourcing
    > ~/.bashrc
    
    # Add core environment sources to .profile (preserve any existing build-time setup)
    cat >> ~/.profile << 'EOF'
# Teaclave TrustZone SDK Environment Setup (added by entrypoint)
source ${TEACLAVE_TOOLCHAIN_BASE}/setup/bootstrap_env
source ${TEACLAVE_TOOLCHAIN_BASE}/environment
export PATH=${TEACLAVE_TOOLCHAIN_BASE}/bin:$PATH
EOF
    
    # Add interactive shell setup to .bashrc (NO profile sourcing to avoid circular dependency)
    cat >> ~/.bashrc << 'EOF'
# Teaclave TrustZone SDK interactive shell setup
EOF

    # Append the unified function definition
    define_switch_config >> ~/.bashrc
}

# Set up the bash profile
setup_bash_profile

# If no command provided, start interactive bash
if [ $# -eq 0 ]; then
    exec /bin/bash -l
else
    # Execute the provided command in a login shell (which sources .profile)
    exec /bin/bash -lc "$*"
fi