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

# Show usage
show_usage() {
    cat << EOF
Usage: TA_DEV_KIT_DIR=<path> OPTEE_CLIENT_EXPORT=<path> $0 [OPTIONS]

Required environment variables:
  TA_DEV_KIT_DIR          Path to OP-TEE OS TA dev kit directory
  OPTEE_CLIENT_EXPORT     Path to OP-TEE client export directory

Options:
  --ta <arch>             TA architecture: aarch64 or arm (default: aarch64)
  --host <arch>           Host architecture for CA and plugins: aarch64 or arm (default: aarch64)
  --std                   Install with std support (default: no-std)
  --ta-install-dir <path> TA installation directory (default: ./tests/shared)
  --ca-install-dir <path> CA installation directory (default: ./tests/shared)
  --plugin-install-dir <path> Plugin installation directory (default: ./tests/shared)
  --help                  Show this help message

Examples:
  # Install for aarch64 in no-std mode
  TA_DEV_KIT_DIR=/path/to/export-ta_arm64 OPTEE_CLIENT_EXPORT=/path/to/export ./build.sh

  # Install for ARM32 in std mode with custom directories
  TA_DEV_KIT_DIR=/path/to/export-ta_arm32 OPTEE_CLIENT_EXPORT=/path/to/export ./build.sh --ta arm --host arm --std --ta-install-dir /target/lib/optee_armtz --ca-install-dir /target/usr/bin

Note: Binaries are installed to './tests/shared' directory by default.
EOF
}

# Parse command line arguments
ARCH_TA="aarch64"  # Default: aarch64
ARCH_HOST="aarch64"  # Default: aarch64
STD=""             # Default: empty (no-std)
TA_INSTALL_DIR=""  # Default: will be set to ./tests/shared if not specified
CA_INSTALL_DIR=""  # Default: will be set to ./tests/shared if not specified
PLUGIN_INSTALL_DIR=""  # Default: will be set to ./tests/shared if not specified

# Parse arguments (support both positional and flag-style)
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            show_usage
            exit 0
            ;;
        --ta)
            ARCH_TA="$2"
            shift 2
            ;;
        --host)
            ARCH_HOST="$2"
            shift 2
            ;;
        --std)
            STD="std"
            shift
            ;;
        --ta-install-dir)
            TA_INSTALL_DIR="$2"
            shift 2
            ;;
        --ca-install-dir)
            CA_INSTALL_DIR="$2"
            shift 2
            ;;
        --plugin-install-dir)
            PLUGIN_INSTALL_DIR="$2"
            shift 2
            ;;
        *)
            # Positional arguments (backward compatibility)
            if [[ -z "${ARCH_TA_SET:-}" ]]; then
                ARCH_TA="$1"
                ARCH_TA_SET=1
            elif [[ -z "${ARCH_HOST_SET:-}" ]]; then
                ARCH_HOST="$1"
                ARCH_HOST_SET=1
            elif [[ "$1" == "std" ]]; then
                STD="std"
            fi
            shift
            ;;
    esac
done

# Validate architecture
if [[ "$ARCH_TA" != "aarch64" && "$ARCH_TA" != "arm" ]]; then
    echo "Error: ARCH_TA must be 'aarch64' or 'arm'"
    exit 1
fi

if [[ "$ARCH_HOST" != "aarch64" && "$ARCH_HOST" != "arm" ]]; then
    echo "Error: ARCH_HOST must be 'aarch64' or 'arm'"
    exit 1
fi

# Check required environment variables
if [ -z "$TA_DEV_KIT_DIR" ]; then
    echo "Error: TA_DEV_KIT_DIR environment variable is not set"
    exit 1
fi

if [ -z "$OPTEE_CLIENT_EXPORT" ]; then
    echo "Error: OPTEE_CLIENT_EXPORT environment variable is not set"
    exit 1
fi

echo "==========================================="
echo "Installing with configuration:"
echo "  ARCH_TA: $ARCH_TA"
echo "  ARCH_HOST: $ARCH_HOST"
echo "  STD: ${STD:-no-std}"
echo "  TA_DEV_KIT_DIR: $TA_DEV_KIT_DIR"
echo "  OPTEE_CLIENT_EXPORT: $OPTEE_CLIENT_EXPORT"
echo "==========================================="

# Step 1: Build cargo-optee tool
echo ""
echo "Step 1: Building cargo-optee tool..."
cd cargo-optee
cargo build --release
CARGO_OPTEE="$(pwd)/target/release/cargo-optee"
cd ..

if [ ! -f "$CARGO_OPTEE" ]; then
    echo "Error: Failed to build cargo-optee"
    exit 1
fi

echo "cargo-optee built successfully: $CARGO_OPTEE"

# Prepare std flag for cargo-optee
STD_FLAG=""
if [ -n "$STD" ]; then
    STD_FLAG="--std"
fi

# Step 2: Install all examples to shared directory
echo ""
echo "Step 2: Installing all examples..."

# Set up installation directories
# Each directory defaults to ./tests/shared if not specified
if [ -z "$TA_INSTALL_DIR" ]; then
    TA_INSTALL_DIR="$(pwd)/tests/shared"
fi

if [ -z "$CA_INSTALL_DIR" ]; then
    CA_INSTALL_DIR="$(pwd)/tests/shared"
fi

if [ -z "$PLUGIN_INSTALL_DIR" ]; then
    PLUGIN_INSTALL_DIR="$(pwd)/tests/shared"
fi

# Create all directories
mkdir -p "$TA_INSTALL_DIR"
mkdir -p "$CA_INSTALL_DIR"
mkdir -p "$PLUGIN_INSTALL_DIR"

echo "Installing binaries to:"
echo "  TAs: $TA_INSTALL_DIR"
echo "  CAs: $CA_INSTALL_DIR"
echo "  Plugins: $PLUGIN_INSTALL_DIR"

EXAMPLES_DIR="$(pwd)/examples"
METADATA_JSON="$EXAMPLES_DIR/metadata.json"

if [ ! -f "$METADATA_JSON" ]; then
    echo "Error: $METADATA_JSON not found"
    exit 1
fi

# Check if jq is available for JSON parsing
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required to parse metadata.json"
    echo "Please install jq: apt-get install jq"
    exit 1
fi

echo "Loading example metadata from $METADATA_JSON..."

# Get all example names
ALL_EXAMPLES=($(jq -r '.examples | keys[]' "$METADATA_JSON"))

if [ -n "$STD" ]; then
    echo "Building in STD mode (std-only + common examples)"
else
    echo "Building in NO-STD mode (no-std-only + common examples)"
fi

CURRENT=0
FAILED_EXAMPLES=""

# Build examples
for EXAMPLE_NAME in "${ALL_EXAMPLES[@]}"; do
    CATEGORY=$(jq -r ".examples[\"$EXAMPLE_NAME\"].category" "$METADATA_JSON")
    
    # Determine if we should build this example
    SHOULD_BUILD=false
    if [ -n "$STD" ]; then
        # STD mode: build std-only and common
        if [[ "$CATEGORY" == "std-only" || "$CATEGORY" == "common" ]]; then
            SHOULD_BUILD=true
        fi
    else
        # NO-STD mode: build no-std-only and common
        if [[ "$CATEGORY" == "no-std-only" || "$CATEGORY" == "common" ]]; then
            SHOULD_BUILD=true
        fi
    fi
    
    if [ "$SHOULD_BUILD" = false ]; then
        continue
    fi
    
    CURRENT=$((CURRENT + 1))
    EXAMPLE_DIR="$EXAMPLES_DIR/$EXAMPLE_NAME"
    
    if [ ! -d "$EXAMPLE_DIR" ]; then
        echo "ERROR: Example directory not found: $EXAMPLE_DIR"
        FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME"
        continue
    fi
    
    echo ""
    echo "=========================================="
    echo "[$CURRENT] Building: $EXAMPLE_NAME ($CATEGORY)"
    echo "=========================================="
    
    # Get TA, CA, and Plugin directories from metadata
    TAS_JSON=$(jq -c ".examples[\"$EXAMPLE_NAME\"].tas" "$METADATA_JSON")
    CAS_JSON=$(jq -c ".examples[\"$EXAMPLE_NAME\"].cas" "$METADATA_JSON")
    PLUGINS_JSON=$(jq -c ".examples[\"$EXAMPLE_NAME\"].plugins // []" "$METADATA_JSON")
    
    # Build all TAs for this example
    TA_COUNT=$(echo "$TAS_JSON" | jq 'length')
    CA_COUNT=$(echo "$CAS_JSON" | jq 'length')
    PLUGIN_COUNT=$(echo "$PLUGINS_JSON" | jq 'length')
    
    echo "→ Found $TA_COUNT TA(s), $CA_COUNT CA(s), and $PLUGIN_COUNT Plugin(s)"
    
    if [ "$TA_COUNT" -gt 0 ]; then
        for ((i=0; i<$TA_COUNT; i++)); do
            TA_DIR=$(echo "$TAS_JSON" | jq -r ".[$i]")
            TA_DIR_FULL_PATH="$EXAMPLES_DIR/$TA_DIR"
            
            if [ ! -d "$TA_DIR_FULL_PATH" ]; then
                echo "ERROR: TA directory not found: $TA_DIR_FULL_PATH"
                FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($TA_DIR)"
                continue
            fi
            
            if [ ! -f "$TA_DIR_FULL_PATH/Cargo.toml" ]; then
                echo "ERROR: Cargo.toml not found in TA directory: $TA_DIR_FULL_PATH"
                FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($TA_DIR)"
                continue
            fi
            
            echo ""
            echo "→ Building TA [$((i+1))/$TA_COUNT]: $TA_DIR"
            
            # Determine STD_FLAG for TA
            TA_STD_FLAG=""
            if [ -n "$STD" ]; then
                # In std mode: always pass --std flag to cargo-optee
                TA_STD_FLAG="--std"
            fi
            
            # Change to TA directory and run cargo-optee without --manifest-path
            cd "$TA_DIR_FULL_PATH"
            
            # Run cargo-optee install and capture both stdout and stderr
            if $CARGO_OPTEE install ta \
                --target-dir "$TA_INSTALL_DIR" \
                --ta-dev-kit-dir "$TA_DEV_KIT_DIR" \
                --arch "$ARCH_TA" \
                $TA_STD_FLAG; then
                echo "  ✓ TA installed successfully"
                # Clean up build artifacts
                $CARGO_OPTEE clean
            else
                echo "  ✗ ERROR: Failed to install TA: $TA_DIR"
                FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($TA_DIR)"
                cd "$EXAMPLES_DIR"  # Return to examples directory
                continue
            fi
            
            # Return to examples directory
            cd "$EXAMPLES_DIR"
        done
    else
        echo "WARNING: No TAs defined for $EXAMPLE_NAME"
    fi
    
    # Build each CA
    CA_INDEX=0
    while [[ "$CA_INDEX" -lt "$CA_COUNT" ]]; do
        CA_DIR=$(echo "$CAS_JSON" | jq -r ".[$CA_INDEX]")
        CA_DIR_FULL_PATH="$EXAMPLES_DIR/$CA_DIR"
        
        echo ""
        echo "→ Building CA [$((CA_INDEX+1))/$CA_COUNT]: $CA_DIR"
        
        if [ ! -d "$CA_DIR_FULL_PATH" ]; then
            echo "ERROR: CA directory not found: $CA_DIR_FULL_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($CA_DIR)"
            CA_INDEX=$((CA_INDEX + 1))
            continue
        fi
        
        if [ ! -f "$CA_DIR_FULL_PATH/Cargo.toml" ]; then
            echo "ERROR: Cargo.toml not found in CA directory: $CA_DIR_FULL_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($CA_DIR)"
            CA_INDEX=$((CA_INDEX + 1))
            continue
        fi
        
        # Change to CA directory and run cargo-optee without --manifest-path
        cd "$CA_DIR_FULL_PATH"
        
        if $CARGO_OPTEE install ca \
            --target-dir "$CA_INSTALL_DIR" \
            --optee-client-export "$OPTEE_CLIENT_EXPORT" \
            --arch "$ARCH_HOST"; then
            echo "  ✓ CA installed successfully"
            # Clean up build artifacts
            $CARGO_OPTEE clean
        else
            echo "  ✗ ERROR: Failed to install CA: $CA_DIR"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($CA_DIR)"
            cd "$EXAMPLES_DIR"  # Return to examples directory
            CA_INDEX=$((CA_INDEX + 1))
            continue
        fi
        
        # Return to examples directory
        cd "$EXAMPLES_DIR"
        CA_INDEX=$((CA_INDEX + 1))
    done
    
    # Build each Plugin
    PLUGIN_INDEX=0
    while [[ "$PLUGIN_INDEX" -lt "$PLUGIN_COUNT" ]]; do
        PLUGIN_DIR=$(echo "$PLUGINS_JSON" | jq -r ".[$PLUGIN_INDEX]")
        PLUGIN_DIR_FULL_PATH="$EXAMPLES_DIR/$PLUGIN_DIR"
        
        echo ""
        echo "→ Building Plugin [$((PLUGIN_INDEX+1))/$PLUGIN_COUNT]: $PLUGIN_DIR"
        
        if [ ! -d "$PLUGIN_DIR_FULL_PATH" ]; then
            echo "ERROR: Plugin directory not found: $PLUGIN_DIR_FULL_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($PLUGIN_DIR)"
            PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
            continue
        fi
        
        if [ ! -f "$PLUGIN_DIR_FULL_PATH/Cargo.toml" ]; then
            echo "ERROR: Cargo.toml not found in Plugin directory: $PLUGIN_DIR_FULL_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($PLUGIN_DIR)"
            PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
            continue
        fi
        
        # Change to Plugin directory and run cargo-optee without --manifest-path
        cd "$PLUGIN_DIR_FULL_PATH"
        
        if $CARGO_OPTEE install plugin \
            --target-dir "$PLUGIN_INSTALL_DIR" \
            --optee-client-export "$OPTEE_CLIENT_EXPORT" \
            --arch "$ARCH_HOST"; then
            echo "  ✓ Plugin installed successfully"
            # Clean up build artifacts
            $CARGO_OPTEE clean
        else
            echo "  ✗ ERROR: Failed to install Plugin: $PLUGIN_DIR"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($PLUGIN_DIR)"
            cd "$EXAMPLES_DIR"  # Return to examples directory
            PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
            continue
        fi
        
        # Return to examples directory
        cd "$EXAMPLES_DIR"
        PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
    done
    
    echo ""
    echo "✓ Example $EXAMPLE_NAME completed successfully"
done

# Summary
echo ""
echo "==========================================="
echo "         INSTALL SUMMARY"
echo "==========================================="
echo ""
echo "Mode:          ${STD:-no-std}"
echo "Architecture:  TA=$ARCH_TA, CA=$ARCH_CA"
echo "Examples:      $CURRENT installed"
echo "Target dir:    $SHARED_DIR"
echo ""

if [ -n "$FAILED_EXAMPLES" ]; then
    echo "❌ INSTALL FAILED"
    echo ""
    echo "Failed components:"
    echo -e "$FAILED_EXAMPLES"
    echo ""
    exit 1
else
    echo "✅ ALL EXAMPLES INSTALLED SUCCESSFULLY!"
    echo ""
fi

