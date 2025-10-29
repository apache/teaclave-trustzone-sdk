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
  --ca <arch>             CA architecture: aarch64 or arm (default: aarch64)
  --std                   Build with std support (default: no-std)
  --clean                 Clean build artifacts before building
  --help                  Show this help message

Examples:
  # Build for aarch64 in no-std mode
  TA_DEV_KIT_DIR=/path/to/export-ta_arm64 OPTEE_CLIENT_EXPORT=/path/to/export ./build.sh

  # Build for ARM32 in std mode
  TA_DEV_KIT_DIR=/path/to/export-ta_arm32 OPTEE_CLIENT_EXPORT=/path/to/export ./build.sh --ta arm --ca arm --std

  # Clean build
  TA_DEV_KIT_DIR=/path/to/export-ta_arm64 OPTEE_CLIENT_EXPORT=/path/to/export ./build.sh --clean
EOF
}

# Parse command line arguments
ARCH_TA="aarch64"  # Default: aarch64
ARCH_CA="aarch64"  # Default: aarch64
STD=""             # Default: empty (no-std)
CLEAN=false        # Default: no clean

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
        --ca)
            ARCH_CA="$2"
            shift 2
            ;;
        --std)
            STD="std"
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        *)
            # Positional arguments (backward compatibility)
            if [[ -z "${ARCH_TA_SET:-}" ]]; then
                ARCH_TA="$1"
                ARCH_TA_SET=1
            elif [[ -z "${ARCH_CA_SET:-}" ]]; then
                ARCH_CA="$1"
                ARCH_CA_SET=1
            elif [[ "$1" == "std" ]]; then
                STD="std"
            elif [[ "$1" == "clean" ]]; then
                CLEAN=true
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

if [[ "$ARCH_CA" != "aarch64" && "$ARCH_CA" != "arm" ]]; then
    echo "Error: ARCH_CA must be 'aarch64' or 'arm'"
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
echo "Building with configuration:"
echo "  ARCH_TA: $ARCH_TA"
echo "  ARCH_CA: $ARCH_CA"
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

# Clean build artifacts if requested
if [ "$CLEAN" = true ]; then
    echo ""
    echo "Cleaning build artifacts..."
    cd optee-teec
    cargo clean
    cd ../optee-utee
    cargo clean
    cd ../optee-utee-build
    cargo clean
    cd ..
    
    # Clean all example directories
    find examples -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
    echo "Build artifacts cleaned"
fi

# Prepare std flag for cargo-optee
STD_FLAG=""
if [ -n "$STD" ]; then
    STD_FLAG="--std"
fi

# Step 2: Build all examples
echo ""
echo "Step 2: Building all examples..."

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
    
    # Get TA, CA, and Plugin paths from metadata
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
            TA_PATH=$(echo "$TAS_JSON" | jq -r ".[$i].path")
            TA_UUID=$(echo "$TAS_JSON" | jq -r ".[$i].uuid")
            
            TA_FULL_PATH="$EXAMPLES_DIR/$TA_PATH"
            UUID_FULL_PATH="$EXAMPLES_DIR/$TA_UUID"
            
            if [ ! -d "$TA_FULL_PATH" ]; then
                echo "ERROR: TA directory not found: $TA_FULL_PATH"
                FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($TA_PATH)"
                continue
            fi
            
            echo ""
            echo "→ Building TA [$((i+1))/$TA_COUNT]: $TA_PATH"
            
            # Determine STD_FLAG for TA
            TA_STD_FLAG=""
            if [ -n "$STD" ]; then
                # In std mode: always pass --std flag to cargo-optee
                # cargo-optee uses --no-default-features so std-only TAs won't fail
                TA_STD_FLAG="--std"
            fi
            
            if $CARGO_OPTEE build ta \
                --path "$TA_FULL_PATH" \
                --ta_dev_kit_dir "$TA_DEV_KIT_DIR" \
                --arch "$ARCH_TA" \
                --uuid_path "$UUID_FULL_PATH" \
                $TA_STD_FLAG 2>&1 | grep -E "(Building TA|Running cargo|Building TA binary|Stripping|Signing|SIGN|TA build completed|Error|error:)"; then
                echo "  ✓ TA built successfully"
            else
                echo "  ✗ ERROR: Failed to build TA: $TA_PATH"
                FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($TA_PATH)"
                continue
            fi
        done
    else
        echo "WARNING: No TAs defined for $EXAMPLE_NAME"
    fi
    
    # Build each CA
    CA_INDEX=0
    while [[ "$CA_INDEX" -lt "$CA_COUNT" ]]; do
        CA_ENTRY=$(echo "$CAS_JSON" | jq -r ".[$CA_INDEX]")
        
        # CAs should be plain strings (not objects)
        CA_PATH="$CA_ENTRY"
        CA_FULL_PATH="$EXAMPLES_DIR/$CA_PATH"
        
        echo ""
        echo "→ Building CA [$((CA_INDEX+1))/$CA_COUNT]: $CA_PATH"
        
        if $CARGO_OPTEE build ca \
            --path "$CA_FULL_PATH" \
            --optee_client_export "$OPTEE_CLIENT_EXPORT" \
            --arch "$ARCH_CA" 2>&1 | grep -E "(Building CA|Running cargo|Building CA binary|CA build completed|Error|error:)"; then
            echo "  ✓ CA built successfully"
        else
            echo "  ✗ ERROR: Failed to build CA: $CA_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($CA_PATH)"
            CA_INDEX=$((CA_INDEX + 1))
            continue
        fi
        
        CA_INDEX=$((CA_INDEX + 1))
    done
    
    # Build each Plugin (PLUGINS_JSON already defined at top)
    PLUGIN_INDEX=0
    while [[ "$PLUGIN_INDEX" -lt "$PLUGIN_COUNT" ]]; do
        PLUGIN_PATH=$(echo "$PLUGINS_JSON" | jq -r ".[$PLUGIN_INDEX].path")
        PLUGIN_UUID_PATH=$(echo "$PLUGINS_JSON" | jq -r ".[$PLUGIN_INDEX].uuid")
        
        PLUGIN_FULL_PATH="$EXAMPLES_DIR/$PLUGIN_PATH"
        PLUGIN_UUID_FULL_PATH="$EXAMPLES_DIR/$PLUGIN_UUID_PATH"
        
        echo ""
        echo "→ Building Plugin [$((PLUGIN_INDEX+1))/$PLUGIN_COUNT]: $PLUGIN_PATH"
        
        if $CARGO_OPTEE build plugin \
            --path "$PLUGIN_FULL_PATH" \
            --optee_client_export "$OPTEE_CLIENT_EXPORT" \
            --arch "$ARCH_CA" \
            --uuid_path "$PLUGIN_UUID_FULL_PATH" 2>&1 | grep -E "(Building Plugin|Running cargo|Building.*binary|Processing plugin|Plugin.*completed|Error|error:)"; then
            echo "  ✓ Plugin built successfully"
        else
            echo "  ✗ ERROR: Failed to build Plugin: $PLUGIN_PATH"
            FAILED_EXAMPLES="$FAILED_EXAMPLES\n  - $EXAMPLE_NAME ($PLUGIN_PATH)"
            PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
            continue
        fi
        
        PLUGIN_INDEX=$((PLUGIN_INDEX + 1))
    done
    
    echo ""
    echo "✓ Example $EXAMPLE_NAME completed successfully"
done

# Summary
echo ""
echo "==========================================="
echo "           BUILD SUMMARY"
echo "==========================================="
echo ""
echo "Mode:          ${STD:-no-std}"
echo "Architecture:  TA=$ARCH_TA, CA=$ARCH_CA"
echo "Examples:      $CURRENT built"
echo ""

if [ -n "$FAILED_EXAMPLES" ]; then
    echo "❌ BUILD FAILED"
    echo ""
    echo "Failed components:"
    echo -e "$FAILED_EXAMPLES"
    echo ""
    exit 1
else
    echo "✅ ALL EXAMPLES BUILT SUCCESSFULLY!"
    echo ""
fi

