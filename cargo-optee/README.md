# cargo-optee

A Cargo subcommand for building OP-TEE Trusted Applications (TAs) and Client Applications (CAs) in Rust.

## Overview

`cargo-optee` simplifies the development workflow for OP-TEE applications by replacing complex Makefiles with a unified, type-safe command-line interface. It handles cross-compilation, custom target specifications, environment setup, and signing automatically.

## High-Level Design

### Architecture

```
                  ┌──────────────────┐
                  │  TA Developer    │
                  │  (CLI input)     │
                  └────────┬─────────┘
                           │
                           ▼
        ┌──────────────────────────────────────────────┐
        │         cargo-optee (this tool)              │
        │                                              │
        │  ┌────────────────────────────────────────┐  │
        │  │  1. Parse CLI & Validate Parameters    │  │
        │  │     - Architecture (aarch64/arm)       │  │
        │  │     - Build mode (std/no-std)          │  │
        │  │     - Build type (TA/CA/PLUGIN)        │  │
        │  └──────────────────┬─────────────────────┘  │
        │                     │                        │
        │  ┌──────────────────▼─────────────────────┐  │
        │  │  2. Setup Build Environment            │  │
        │  │     - Set environment variables        │  │
        │  │     - Configure cross-compiler         │  │
        │  └──────────────────┬─────────────────────┘  │
        │                     │                        │
        │  ┌──────────────────▼─────────────────────┐  │
        │  │  3. Execute Build Pipeline             │  │
        │  │     - Run clippy (linting)             │  │
        │  │     - Build binary: cargo/xargo + gcc  │  │
        │  │     - Strip symbols: objcopy           │  │
        │  │     - Sign TA: Python script (TA only) │  │
        │  └──────────────────┬─────────────────────┘  │
        │                     │                        │
        └─────────────────────┼────────────────────────┘
                              │
                              ▼
        ┌──────────────────────────────────────────────┐
        │    Low-Level Tools (dependencies)            │
        │                                              │
        │    - cargo/xargo: Rust compilation           │
        │    - gcc: Linking with OP-TEE libraries      │
        │    - objcopy: Symbol stripping               │
        │    - Python script: TA signing (TA only)     │
        │                                              │
        └──────────────────────────────────────────────┘
```

## Quick Start

### Installation

Assume developers have Rust, Cargo, and the gcc toolchain installed and added to PATH (the guide is in future plan). Then install `cargo-optee` using Cargo:

```bash
cargo install cargo-optee
```

### Project Structure

Cargo-optee expects the following project structure by default.

```
project/
├── uuid.txt           # TA UUID
├── ta/                # Trusted Application
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── build.rs       # Build script
├── host/              # Client Application (host)
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
└── proto/             # Shared definitions such as TA command IDs and TA UUID
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

See examples in the SDK for reference, such as `hello_world-rs`.
The `cargo new` command (planned, not yet available) will generate a project template with this structure. For now, copy an existing example as a starting point.

### Build Commands

#### Build Trusted Application (TA)

```bash
cargo-optee build ta \
  --ta_dev_kit_dir <PATH> \
  [--path <PATH>] \
  [--arch aarch64|arm] \
  [--std] \
  [--signing_key <PATH>] \
  [--uuid_path <PATH>] \
  [--debug]
```

**Required:**
- `--ta_dev_kit_dir <PATH>`: Path to OP-TEE TA development kit (available after building OP-TEE OS), user must provide this for building TAs.

**Optional:**
- `--path <PATH>`: Path to TA project directory (default: `.`)
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
  - `aarch64`: ARM 64-bit architecture
  - `arm`: ARM 32-bit architecture
- `--std`: Build with std support (uses xargo and custom target)
- `--signing_key <PATH>`: Path to signing key (default: `<ta_dev_kit_dir>/keys/default_ta.pem`)
- `--uuid_path <PATH>`: Path to UUID file (default: `../uuid.txt`)
- `--debug`: Build in debug mode (default: release mode)

**Example:**
```bash
# Build aarch64 TA with std support
cargo-optee build ta \
  --ta_dev_kit_dir /opt/optee/export-ta_arm64 \
  --path ./examples/hello_world-rs/ta \
  --arch aarch64 \
  --std

# Build arm TA without std (no-std)
cargo-optee build ta \
  --ta_dev_kit_dir /opt/optee/export-ta_arm32 \
  --path ./ta \
  --arch arm
```

**Output:**
- TA binary: `target/<target-triple>/release/<uuid>.ta`
- Intermediate files in `target/` directory

#### Build Client Application (CA)

```bash
cargo-optee build ca \
  --optee_client_export <PATH> \
  [--path <PATH>] \
  [--arch aarch64|arm] \
  [--debug]
```

**Required:**
- `--optee_client_export <PATH>`: Path to OP-TEE client library directory (available after building OP-TEE client), user must provide this for building CAs.

**Optional:**
- `--path <PATH>`: Path to CA project directory (default: `.`)
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
- `--debug`: Build in debug mode (default: release mode)

**Example:**
```bash
# Build aarch64 client application
cargo-optee build ca \
  --optee_client_export /opt/optee/export-client \
  --path ./examples/hello_world-rs/host \
  --arch aarch64
```

**Output:**
- CA binary: `target/<target-triple>/release/<binary-name>`

#### Build Plugin

We have one example for plugin: `supp_plugin-rs/plugin`.

```bash
cargo-optee build plugin \
  --optee_client_export <PATH> \
  --uuid_path <PATH> \
  [--path <PATH>] \
  [--arch aarch64|arm] \
  [--debug]
```

**Required:**
- `--optee_client_export <PATH>`: Path to OP-TEE client library directory (available after building OP-TEE client), user must provide this for building plugins.
- `--uuid_path <PATH>`: Path to UUID file for naming the plugin

**Optional:**
- `--path <PATH>`: Path to plugin project directory (default: `.`)
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
- `--debug`: Build in debug mode (default: release mode)

**Example:**
```bash
# Build aarch64 plugin
cargo-optee build plugin \
  --optee_client_export /opt/optee/export-client \
  --path ./examples/supp_plugin-rs/plugin \
  --uuid_path ./examples/supp_plugin-rs/plugin_uuid.txt \
  --arch aarch64
```

**Output:**
- Plugin binary: `target/<target-triple>/release/<uuid>.plugin.so`

### Usage Workflows (including future design)

#### Development/Emulation Environment

For development and emulation, developers would like to build the one project and deploy to a target filesystem (e.g. QEMU shared folder) quickly. Frequent builds and quick rebuilds are common. 

```bash
# 1. Create new project (future)
cargo-optee new my_app
cd my_app

# 2. Build TA and CA
cargo-optee build ta \
  --ta_dev_kit_dir $TA_DEV_KIT_DIR \
  --path ./ta \
  --arch aarch64 \
  --std

cargo-optee build ca \
  --optee_client_export $OPTEE_CLIENT_EXPORT \
  --path ./host \
  --arch aarch64

# 3. Install to specific folder (future), e.g. QEMU shared folder for emulation
cargo-optee install --target /tmp/qemu-shared-folder
```

#### Production/CI Environment

For production and CI environments, artifacts should be cleaned up after successful builds. It can help to avoid storage issues on CI runners.

**Automated Build Pipeline:**
```bash
#!/bin/bash
# CI build script

set -e

# Build TA (release mode)
cargo-optee build ta \
  --ta_dev_kit_dir $TA_DEV_KIT_DIR \
  --path ./ta \
  --arch aarch64 \
  --std \
  --signing_key ./keys/production.pem

# Build CA (release mode)
cargo-optee build ca \
  --optee_client_export $OPTEE_CLIENT_EXPORT \
  --path ./host \
  --arch aarch64

# Install to staging area (future)
cargo-optee install --target ./dist

# Clean build artifacts to save space (future)
cargo-optee clean --all
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `build ta` | ✅ Implemented | Supports aarch64/arm, std/no-std |
| `build ca` | ✅ Implemented | Supports aarch64/arm |
| `build plugin` | ✅ Implemented | Supports aarch64/arm, builds shared library plugins |
| `new` | ⏳ Planned | Project scaffolding |
| `install` | ⏳ Planned | Deploy to target filesystem |
| `clean` | ⏳ Planned | Remove build artifacts |
| `clean` | ⏳ Planned | Remove build artifacts |

-----
## Appendix

### Complete Parameter Reference

#### Command Convention: User Input to Cargo Commands

##### Example 1: Build aarch64 no-std TA

**User Input:**
```bash
cargo-optee build ta \
  --ta_dev_kit_dir /opt/optee/export-ta_arm64 \
  --path ./ta \
  --arch aarch64
```

**cargo-optee translates to:**
```bash
# 1. Clippy
cd ./ta
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64 \
RUSTFLAGS="-C panic=abort" \
cargo clippy --target aarch64-unknown-linux-gnu --release

# 2. Build
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64 \
RUSTFLAGS="-C panic=abort" \
cargo build --target aarch64-unknown-linux-gnu --release \
  --config target.aarch64-unknown-linux-gnu.linker="aarch64-linux-gnu-gcc"

# 3. Strip
aarch64-linux-gnu-objcopy --strip-unneeded \
  target/aarch64-unknown-linux-gnu/release/ta \
  target/aarch64-unknown-linux-gnu/release/stripped_ta

# 4. Sign
python3 /opt/optee/export-ta_arm64/scripts/sign_encrypt.py \
  --uuid <uuid-from-file> \
  --key /opt/optee/export-ta_arm64/keys/default_ta.pem \
  --in target/aarch64-unknown-linux-gnu/release/stripped_ta \
  --out target/aarch64-unknown-linux-gnu/release/<uuid>.ta
```

#### Example 2: Build arm std TA

**User Input:**
```bash
cargo-optee build ta \
  --ta_dev_kit_dir /opt/optee/export-ta_arm32 \
  --path ./ta \
  --arch arm \
  --std
```

**cargo-optee translates to:**
```bash
# 1. Clippy
cd ./ta
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm32 \
RUSTFLAGS="-C panic=abort" \
RUST_TARGET_PATH=/tmp/cargo-optee-XXXXX \
xargo clippy --target arm-unknown-optee --features std --release

# 2. Build
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm32 \
RUSTFLAGS="-C panic=abort" \
RUST_TARGET_PATH=/tmp/cargo-optee-XXXXX \
xargo build --target arm-unknown-optee --features std --release \
  --config target.arm-unknown-optee.linker="arm-linux-gnueabihf-gcc"

# 3. Strip
arm-linux-gnueabihf-objcopy --strip-unneeded \
  target/arm-unknown-optee/release/ta \
  target/arm-unknown-optee/release/stripped_ta

# 4. Sign
python3 /opt/optee/export-ta_arm32/scripts/sign_encrypt.py \
  --uuid <uuid-from-file> \
  --key /opt/optee/export-ta_arm32/keys/default_ta.pem \
  --in target/arm-unknown-optee/release/stripped_ta \
  --out target/arm-unknown-optee/release/<uuid>.ta
```

**Note:** `/tmp/cargo-optee-XXXXX` is a temporary directory containing the embedded `arm-unknown-optee.json` target specification.

##### Example 3: Build aarch64 CA (Client Application)

**User Input:**
```bash
cargo-optee build ca \
  --optee_client_export /opt/optee/export-client \
  --path ./host
```

**cargo-optee translates to:**
```bash
# 1. Clippy
cd ./host
OPTEE_CLIENT_EXPORT=/opt/optee/export-client \
cargo clippy --target aarch64-unknown-linux-gnu

# 2. Build
OPTEE_CLIENT_EXPORT=/opt/optee/export-client \
cargo build --target aarch64-unknown-linux-gnu --release \
  --config target.aarch64-unknown-linux-gnu.linker="aarch64-linux-gnu-gcc"

# 3. Strip
aarch64-linux-gnu-objcopy --strip-unneeded \
  target/aarch64-unknown-linux-gnu/release/<binary> \
  target/aarch64-unknown-linux-gnu/release/<binary>
```

#### Build Command Convention: Cargo to Low-Level Tools

This section explains how cargo orchestrates low-level tools to build the TA ELF binary. We use an aarch64 no-std TA as an example.

**Dependency Structure:**
```
ta
├── depends on: optee_utee (Rust API for OP-TEE TAs)
│   └── depends on: optee_utee_sys (FFI bindings to OP-TEE C API)
│       └── build.rs: outputs cargo:rustc-link-* directives
│           Links with C libraries from TA_DEV_KIT_DIR/lib/:
│           - libutee.a (OP-TEE user-space TA API)
│           - libutils.a (utility functions)
│           - libmbedtls.a (crypto library)
└── build.rs: uses optee_utee_build crate to:
    - Configure TA properties (UUID, stack size, etc.)
    - Generate TA header file (user_ta_header.rs)
    - Output link directives
```

**Build Flow:**

**Step 1: cargo-optee invokes cargo**

(As shown in the previous section)
```bash
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64 \
RUSTFLAGS="-C panic=abort" \
cargo build --target aarch64-unknown-linux-gnu --release \
  --config target.aarch64-unknown-linux-gnu.linker="aarch64-linux-gnu-gcc"
```

**Step 2: cargo prepares environment and invokes build scripts**

Cargo automatically sets these environment variables:
- `TARGET=aarch64-unknown-linux-gnu`
- `PROFILE=release`
- `OUT_DIR=target/aarch64-unknown-linux-gnu/release/build/ta-<hash>/out`
- `RUSTC_LINKER=aarch64-linux-gnu-gcc` (from `--config` flag)

Cargo inherits from cargo-optee:
- `TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64`
- `RUSTFLAGS="-C panic=abort"`

Cargo then executes build scripts in dependency order to set up the build directives:

1. **`optee_utee_sys/build.rs`**
   - Requires: `TA_DEV_KIT_DIR`
   - Outputs `cargo:rustc-link-*` directives to link C libraries:
     ```
     cargo:rustc-link-search={TA_DEV_KIT_DIR}/lib
     cargo:rustc-link-lib=static=utee
     cargo:rustc-link-lib=static=utils
     cargo:rustc-link-lib=static=mbedtls
     ```

2. **`ta/build.rs`** → calls **`optee_utee_build`** crate
   - Requires: `TA_DEV_KIT_DIR`, `TARGET`, `OUT_DIR`, `RUSTC_LINKER`
   - Optional: `CARGO_PKG_VERSION`, `CARGO_PKG_DESCRIPTION` (for automatic TA config)
   - Actions:
     1. Generates TA manifest (`user_ta_header.rs`) with TA properties
     2. Outputs linker directives based on target architecture and linker type

**Step 3: rustc compiles Rust source code**

Rustc receives:
- Target triple: `--target aarch64-unknown-linux-gnu`
- Compiler flags: `-C panic=abort` (from `RUSTFLAGS`)
- Profile: `release`
- All link directives from build scripts

Produces: `.rlib` files and object files (`.o`)

**Step 4: gcc linker links final binary**

The linker (specified by `RUSTC_LINKER=aarch64-linux-gnu-gcc`) links:
- Rust object files (`.o`)
- OP-TEE C static libraries: `libutee.a`, `libutils.a`, `libmbedtls.a`
- Using linker script: `ta.lds`

**Output:** ELF binary at `target/aarch64-unknown-linux-gnu/release/ta`
