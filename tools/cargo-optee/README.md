# cargo-optee

A Cargo subcommand for building OP-TEE Trusted Applications (TAs) and Client
Applications (CAs) in Rust.

## Overview

`cargo-optee` simplifies the development workflow for OP-TEE applications. SDK
example Makefiles invoke it for clippy, cross-compilation, stripping, and
signing; you can also call the same CLI directly.

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
        │  │  3. Execute command                    │  │
        │  │     clippy: cargo fmt + clippy         │  │
        │  │     build: cargo + strip + sign TA     │  │
        │  └──────────────────┬─────────────────────┘  │
        │                     │                        │
        └─────────────────────┼────────────────────────┘
                              │
                              ▼
        ┌──────────────────────────────────────────────┐
        │    Low-Level Tools (dependencies)            │
        │                                              │
        │    - cargo: Rust compilation                 │
        │    - gcc: Linking with OP-TEE libraries      │
        │    - objcopy: Symbol stripping               │
        │    - Python script: TA signing (TA only)     │
        │                                              │
        └──────────────────────────────────────────────┘
```

## Quick Start

### Installation

Assume developers have Rust, Cargo, and the gcc toolchain installed and added to
PATH (the guide is in future plan). Then install `cargo-optee` using Cargo:

```bash
cargo install cargo-optee
```

### Quick Build for Hello World

This section provides a quick start guide for building the Hello World example
using `cargo-optee`. Before proceeding, ensure you have set up the Docker
development environment. For detailed instructions on setting up the Docker
environment, refer to the [QEMU emulation
guide](../../docs/emulate-and-dev-in-docker.md).

#### Prerequisites

First, pull and run the Docker image. We provide two types of images:
- **no-std environment**: For building TAs without the standard library
  (recommended for quick start)
- **std environment**: For building TAs with Rust standard library support

**Pull and start the no-std development environment:**
```bash
# Pull the pre-built development environment image
docker pull teaclave/teaclave-trustzone-emulator-nostd-expand-memory:latest

# Start the development container
docker run -it --rm \
  --name teaclave_dev_env \
  -v $(pwd):/root/teaclave_sdk_src \
  -w /root/teaclave_sdk_src \
  teaclave/teaclave-trustzone-emulator-nostd-expand-memory:latest
```

> 📖 **Note**: If you need Rust standard library (std) support, refer to the
> [Docker development environment guide with std
> support](../../docs/emulate-and-dev-in-docker-std.md) and use the
> `teaclave/teaclave-trustzone-emulator-std-expand-memory:latest` image.

#### Build Steps

**1. Work from the repository root**

Inside the Docker container, all paths below are relative to the repository
root. The Hello World example is split across two crates:
- TA crate: `examples/ta/hello_world-rs/`
- CA crate: `examples/ca/hello_world-rs/`

**2. Build the Trusted Application (TA)**

Build the TA using `cargo-optee`. In the Docker environment, the OP-TEE TA
development kit is typically located at
`/opt/teaclave/optee/optee_os/out/arm-plat-vexpress/export-ta_arm64`:

```bash
# Build aarch64 no-std TA (default configuration)
cargo-optee build ta \
  --manifest-path examples/ta/hello_world-rs/Cargo.toml \
  --ta-dev-kit-dir /opt/teaclave/optee/optee_os/out/arm-plat-vexpress/export-ta_arm64 \
  --arch aarch64 \
  --no-std
```

**3. Build the Client Application (CA)**

Build the client application:
```bash
# Build aarch64 CA
cargo-optee build ca \
  --manifest-path examples/ca/hello_world-rs/Cargo.toml \
  --optee-client-export /opt/teaclave/optee/optee_client/export_arm64 \
  --arch aarch64
```

> 💡 **Tip**: If you configure metadata in your `Cargo.toml` file (see
> [Configuration System](#configuration-system)), you can simplify the command
> by only specifying `--manifest-path`:
> ```bash
> cargo-optee build ca --manifest-path examples/ca/hello_world-rs/Cargo.toml
> ```

#### Build Output

After a successful build, you can find the generated files at the following
locations:

- **TA binary**:
  `examples/ta/target/aarch64-unknown-linux-gnu/release/133af0ca-bdab-11eb-9130-43bf7873bf67.ta`
- **CA binary**:
  `examples/ca/target/aarch64-unknown-linux-gnu/release/hello_world-rs`

#### Next Steps

After building, you can:
1. Use the `sync_to_emulator` command to sync build artifacts to the emulator
   environment
2. Start the QEMU emulator for testing
3. Refer to the [QEMU emulation guide](../../docs/emulate-and-dev-in-docker.md) for
   complete development and testing workflows

> 💡 **Tip**: If you configure metadata in `Cargo.toml` (see [Configuration
> System](#configuration-system)), you can simplify build commands by only
> specifying `--manifest-path`.

## Configuration System

`cargo-optee` uses a flexible configuration system with the following priority
(highest to lowest):

1. **Command Line Arguments** - Direct CLI flags override everything (see [Build
   through CLI](#build-through-cli))
2. **Cargo.toml Metadata** - Project-specific configuration in
   `[package.metadata.optee.*]` sections (see [Build through
   metadata](#build-through-metadata))
3. **Environment variables** - Used when CLI and metadata omit a value:
   - `TA_DEV_KIT_DIR`, `OPTEE_CLIENT_EXPORT` for kit / client export paths
   - `TA_SIGN_KEY` for the TA signing key
   - `CROSS_COMPILE` for the toolchain prefix (no CLI or metadata equivalent;
     otherwise derived from `--arch`)
4. **Defaults** - Built-in sensible defaults (for example
   `<ta-dev-kit-dir>/keys/default_ta.pem`)

This allows projects to define their standard configuration in `Cargo.toml`
while still permitting CLI overrides for specific builds. Make wrappers pass
`CROSS_COMPILE` and `TA_SIGN_KEY` through the environment.

### Project Structure

Cargo-optee expects the following project structure by default.

```
project/
├── ta/                # Trusted Application
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── build.rs       # Build script (optional; UUID lives in the TA header)
├── host/              # Client Application (host)
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
└── proto/             # Shared definitions such as TA command IDs and TA UUID
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

The TA UUID is defined in the TA's own header (`ta_head.uuid`) and, if you use a
shared proto crate, as a Rust constant consumed by the CA and the TA build
script. `cargo-optee` parses the UUID from the unsigned ELF after the crate is
built. The same rule applies to plugins (`plugin_method.uuid`). Shared proto
crates and `optee-utee-build` / `optee-teec-build` are optional development
helpers — UUID extraction depends only on the OP-TEE 4.10.0 ABI.

See examples in the SDK for reference, such as `hello_world-rs`. Note that in
this SDK's `examples/` directory the layout is split by side instead of by
project: the TA crate lives in `examples/ta/hello_world-rs/`, the CA crate lives
in `examples/ca/hello_world-rs/` (named `ca/` rather than `host/`), and the
shared definitions live in the workspace crate `examples/ta/proto`. The `cargo
new` command (planned, not yet available) will generate a project template with
this structure. For now, copy an existing example as a starting point.

### Usage Workflows

Implemented commands are `build`, `clippy`, `install`, `clean`, and
`inspect-uuid`. `cargo-optee new` and a single `install --target` / `clean --all`
that cover a whole project are not implemented; use the per-component commands
below.

#### Development/Emulation Environment

For development and emulation, developers would like to build the one project
and deploy to a target filesystem (e.g. QEMU shared folder) quickly. Frequent
builds and quick rebuilds are common.

**Using CLI arguments:**
```bash
# 1. Create new project (future)
cargo-optee new my_app
cd my_app

# 2. Build TA and CA
cargo-optee build ta \
  --ta-dev-kit-dir $TA_DEV_KIT_DIR \
  --manifest-path ./ta/Cargo.toml \
  --arch aarch64 \
  --std

cargo-optee build ca \
  --optee-client-export $OPTEE_CLIENT_EXPORT \
  --manifest-path ./host/Cargo.toml \
  --arch aarch64

# 3. Install (copies the built TA/CA into --target-dir, resolved from the
#    process cwd — not the crate directory)
cargo-optee install ta \
  --target-dir /tmp/qemu-shared-folder \
  --ta-dev-kit-dir $TA_DEV_KIT_DIR \
  --manifest-path ./ta/Cargo.toml
cargo-optee install ca \
  --target-dir /tmp/qemu-shared-folder \
  --optee-client-export $OPTEE_CLIENT_EXPORT \
  --manifest-path ./host/Cargo.toml
```

**Using metadata configuration:**
```bash
# 1. Configure once in Cargo.toml files, then simple builds
cd ta && cargo-optee build ta
cd ../host && cargo-optee build ca

# 2. Override specific parameters when needed
cd ta && cargo-optee build ta --debug  # Override to debug build
cd host && cargo-optee build ca --arch arm  # Override architecture
```

#### Production/CI Environment

For production and CI environments, artifacts should be cleaned up after
successful builds. It can help to avoid storage issues on CI runners.

**Automated Build Pipeline:**
```bash
#!/bin/bash
# CI build script

set -e

# Build TA (release mode)
cargo-optee build ta \
  --ta-dev-kit-dir $TA_DEV_KIT_DIR \
  --manifest-path ./ta/Cargo.toml \
  --arch aarch64 \
  --std \
  --signing-key ./keys/production.pem

# Build CA (release mode)
cargo-optee build ca \
  --optee-client-export $OPTEE_CLIENT_EXPORT \
  --manifest-path ./host/Cargo.toml \
  --arch aarch64

# Install (implemented: install ta|ca|plugin --target-dir <DIR>)
cargo-optee install ta --target-dir ./dist --manifest-path ./ta/Cargo.toml
cargo-optee install ca --target-dir ./dist --manifest-path ./host/Cargo.toml

# Clean the crate (implemented: cargo-optee clean [--manifest-path ...])
cargo-optee clean --manifest-path ./ta/Cargo.toml
```

### Build through CLI

#### Build Trusted Application (TA)

```bash
cargo-optee build ta \
  [--ta-dev-kit-dir <PATH>] \
  [--manifest-path <PATH>] \
  [--arch aarch64|arm] \
  [--std] \
  [--no-std] \
  [--signing-key <PATH>] \
  [--debug]
```

**Optional:**

- `--ta-dev-kit-dir <PATH>`: Path to the OP-TEE TA development kit. If omitted,
  cargo-optee reads it from Cargo.toml metadata, then `TA_DEV_KIT_DIR`.
- `--manifest-path <PATH>`: Path to Cargo.toml manifest file
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
  - `aarch64`: ARM 64-bit architecture
  - `arm`: ARM 32-bit architecture
  The cross-compiler prefix is derived from this; set `CROSS_COMPILE` in the
  environment to override (there is no CLI flag).
- `--std`: Build with std support (uses `cargo -Z build-std` and custom target)
- `--no-std`: Build without std support (mutually exclusive with --std)
- `--signing-key <PATH>`: Path to signing key (fallback: Cargo.toml metadata,
  then `TA_SIGN_KEY`, then `<ta-dev-kit-dir>/keys/default_ta.pem`)
- `--debug`: Force a debug build. If omitted, Cargo.toml metadata is used
  (default: release)

The UUID used for signing and the `<uuid>.ta` name is parsed from the unsigned
TA ELF (`ta_head.uuid`).

**Example:**
```bash
# Build aarch64 TA with std support
cargo-optee build ta \
  --ta-dev-kit-dir /opt/optee/export-ta_arm64 \
  --manifest-path ./examples/ta/hello_world-rs/Cargo.toml \
  --arch aarch64 \
  --std

# Build arm TA without std (no-std)
cargo-optee build ta \
  --ta-dev-kit-dir /opt/optee/export-ta_arm32 \
  --manifest-path ./ta/Cargo.toml \
  --arch arm
  --no-std

# Build TA with Cargo.toml metadata or TA_DEV_KIT_DIR configuration
cargo-optee build ta \
  --manifest-path ./ta/Cargo.toml

# Build TA using the OP-TEE environment variable
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64 cargo-optee build ta
```

**Output:**
- TA binary: `target/<target-triple>/release/<uuid>.ta`
- Intermediate files in `target/` directory

#### Build Client Application (CA)

```bash
cargo-optee build ca \
  [--optee-client-export <PATH>] \
  [--manifest-path <PATH>] \
  [--arch aarch64|arm] \
  [--debug]
```

**Optional:**

- `--optee-client-export <PATH>`: Path to the OP-TEE client export directory.
  If omitted, cargo-optee reads it from Cargo.toml metadata, then
  `OPTEE_CLIENT_EXPORT`.
- `--manifest-path <PATH>`: Path to Cargo.toml manifest file
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
- `--debug`: Force a debug build. If omitted, Cargo.toml metadata is used
  (default: release)

**Example:**
```bash
# Build aarch64 client application
cargo-optee build ca \
  --optee-client-export /opt/optee/export-client \
  --manifest-path ./examples/ca/hello_world-rs/Cargo.toml \
  --arch aarch64

# Build CA using the environment variable expected by optee-teec-sys
OPTEE_CLIENT_EXPORT=/opt/optee/export-client cargo-optee build ca
```

**Output:**
- CA binary: `target/<target-triple>/release/<binary-name>`

#### Build Plugin

We have one example for plugin: `examples/ca/supp_plugin-rs-plugin`.

```bash
cargo-optee build plugin \
  [--optee-client-export <PATH>] \
  [--manifest-path <PATH>] \
  [--arch aarch64|arm] \
  [--debug]
```

**Optional:**

- `--optee-client-export <PATH>`: Path to the OP-TEE client export directory.
  If omitted, cargo-optee reads it from Cargo.toml metadata, then
  `OPTEE_CLIENT_EXPORT`.
- `--manifest-path <PATH>`: Path to Cargo.toml manifest file
- `--arch <ARCH>`: Target architecture (default: `aarch64`)
- `--debug`: Force a debug build. If omitted, Cargo.toml metadata is used
  (default: release)

The plugin is copied to `<uuid>.plugin.so` using the UUID parsed from the
dynamic `plugin_method` export.

**Example:**
```bash
# Build aarch64 plugin
cargo-optee build plugin \
  --optee-client-export /opt/optee/export-client \
  --manifest-path ./examples/ca/supp_plugin-rs-plugin/Cargo.toml \
  --arch aarch64
```

**Output:**
- Plugin binary: `target/<target-triple>/release/<uuid>.plugin.so`

#### Inspect UUID

```bash
cargo-optee inspect-uuid --kind ta --elf path/to/ta.elf
cargo-optee inspect-uuid --kind plugin --elf path/to/plugin.so
```

Prints the UUID to stdout. Same parser as `build ta` / `build plugin`.

#### Clippy

Lint only. Same target and environment as the corresponding `build` command;
does not compile the final binary, strip, or sign, and does not require a
signing key.

```bash
cargo-optee clippy ta \
  --manifest-path examples/ta/hello_world-rs/Cargo.toml \
  --arch aarch64 \
  --no-std

cargo-optee clippy ca --manifest-path examples/ca/hello_world-rs/Cargo.toml
cargo-optee clippy plugin --manifest-path examples/ca/supp_plugin-rs-plugin/Cargo.toml
```

Example Makefiles keep the original target split: `make clippy` calls this
command; `make ta` / `make host` depend on it and then call `build`.

`make install` delegates to `cargo-optee install` for each selected example;
`STD` / `TARGET_TA` select the std or no-std set. `O`, `bindir`, and `libdir`
control the installation layout. A component can also be installed with
`make -C examples/ta/hello_world-rs install INSTALL_DIR=/path/to/destination`.
`make emulate` uses the same install command with the `ta` or `host` directory
under `QEMU_HOST_SHARE_DIR`. Makefiles do not infer Cargo artifact paths or
scan target directories for previously built binaries.

### Build through metadata

#### Trusted Application (TA) Metadata

Configure TA builds in your `Cargo.toml`:

```toml
[package.metadata.optee.ta]
arch = "aarch64"                    # Target architecture: "aarch64" | "arm" (optional, default: "aarch64")
debug = false                       # Debug build: true | false (optional, default: false)
std = false                         # Use std library: true | false (optional, default: false)
# Architecture-specific configuration (omitted architectures default to null/unsupported)
ta-dev-kit-dir = { aarch64 = "/opt/optee/export-ta_arm64", arm = "/opt/optee/export-ta_arm32" }
signing-key = "/path/to/key.pem"    # optional; else TA_SIGN_KEY, else ta-dev-kit/keys/default_ta.pem
```

**Allowed entries:**

- `arch`: Target architecture (`"aarch64"` or `"arm"`)
- `debug`: Build in debug mode (`true` or `false`) 
- `std`: Enable std library support (`true` or `false`)
- `ta-dev-kit-dir`: Architecture-specific paths to the TA development kit. If
  omitted, `TA_DEV_KIT_DIR` is used.
- `signing-key`: Path to signing key file. If omitted, `TA_SIGN_KEY` then
  `<ta-dev-kit-dir>/keys/default_ta.pem`.

The TA UUID is taken from the TA header after the crate is built.

#### Client Application (CA) Metadata

Configure CA builds in your `Cargo.toml`:

```toml
[package.metadata.optee.ca]
arch = "aarch64"                    # Target architecture: "aarch64" | "arm" (optional, default: "aarch64")
debug = false                       # Debug build: true | false (optional, default: false)
# Architecture-specific configuration
# if your CA only supports aarch64, you can omit arm
optee-client-export = { aarch64 = "/opt/optee/export-client_arm64" } 
```

**Allowed entries:**

- `arch`: Target architecture (`"aarch64"` or `"arm"`)
- `debug`: Build in debug mode (`true` or `false`)
- `optee-client-export`: Architecture-specific paths to the OP-TEE client
  export. If omitted, `OPTEE_CLIENT_EXPORT` is used.

#### Plugin Metadata

Configure plugin builds in your `Cargo.toml`:

```toml
[package.metadata.optee.plugin]
arch = "aarch64"                    # Target architecture: "aarch64" | "arm" (optional, default: "aarch64")  
debug = false                       # Debug build: true | false (optional, default: false)
# Architecture-specific configuration
optee-client-export = { aarch64 = "/opt/optee/export-client_arm64", arm = "/opt/optee/export-client_arm32" }
```

**Allowed entries:**

- `arch`: Target architecture (`"aarch64"` or `"arm"`)
- `debug`: Build in debug mode (`true` or `false`)
- `optee-client-export`: Architecture-specific paths to OP-TEE client export
  (falls back to `OPTEE_CLIENT_EXPORT`)

The plugin UUID is parsed from the dynamically exported `struct plugin_method`.

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| `build ta` | ✅ Implemented | cargo + strip + sign; aarch64/arm, std/no-std |
| `build ca` | ✅ Implemented | Supports aarch64/arm |
| `build plugin` | ✅ Implemented | Supports aarch64/arm, builds shared library plugins |
| `clippy ta/ca/plugin` | ✅ Implemented | cargo fmt + clippy only |
| `inspect-uuid` | ✅ Implemented | Parse UUID from an unsigned TA ELF or plugin `.so` |
| `install ta/ca/plugin` | ✅ Implemented | Copy the built artifact to `--target-dir` (cwd-relative) |
| `clean` | ✅ Implemented | `cargo clean` for one crate (`--manifest-path`) |
| `new` | ⏳ Planned | Project scaffolding |

-----
## Appendix

### Complete Parameter Reference

#### Command Convention: User Input to Cargo Commands

##### Example 1: Build aarch64 no-std TA

**User Input:**
```bash
cargo-optee build ta \
  --ta-dev-kit-dir /opt/optee/export-ta_arm64 \
  --manifest-path ./ta/Cargo.toml \
  --arch aarch64
```

**cargo-optee translates to:**
```bash
# cargo-optee does not chdir the process; each command uses current_dir +
# --manifest-path pointing at the crate.
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm64 \
RUSTFLAGS="-C panic=abort" \
cargo build --manifest-path ./ta/Cargo.toml --target aarch64-unknown-linux-gnu --release \
  --config target.aarch64-unknown-linux-gnu.linker="aarch64-linux-gnu-gcc"

# 2. Strip (unique tempfile, then discarded after signing)
aarch64-linux-gnu-objcopy --strip-unneeded \
  target/aarch64-unknown-linux-gnu/release/ta \
  target/aarch64-unknown-linux-gnu/release/stripped_ta.<unique>

# 3. Sign (unique .ta.partial, then rename onto <uuid>.ta)
python3 /opt/optee/export-ta_arm64/scripts/sign_encrypt.py \
  --uuid <uuid-parsed-from-stripped-elf> \
  --key /opt/optee/export-ta_arm64/keys/default_ta.pem \
  --in target/aarch64-unknown-linux-gnu/release/stripped_ta.<unique> \
  --out target/aarch64-unknown-linux-gnu/release/<uuid>.ta.<unique>.partial
```

#### Example 2: Build arm std TA

**User Input:**
```bash
cargo-optee build ta \
  --ta-dev-kit-dir /opt/optee/export-ta_arm32 \
  --manifest-path ./ta/Cargo.toml \
  --arch arm \
  --std
```

**cargo-optee translates to:**
```bash
TA_DEV_KIT_DIR=/opt/optee/export-ta_arm32 \
RUSTFLAGS="-C panic=abort" \
RUST_TARGET_PATH=/tmp/cargo-optee-XXXXX \
__CARGO_TESTS_ONLY_SRC_ROOT=/path/to/rust/library \
cargo -Z build-std=std,panic_abort -Z json-target-spec build \
  --manifest-path ./ta/Cargo.toml --target arm-unknown-optee --features std --release \
  --config target.arm-unknown-optee.linker="arm-linux-gnueabihf-gcc"

# 2. Strip (unique tempfile)
arm-linux-gnueabihf-objcopy --strip-unneeded \
  target/arm-unknown-optee/release/ta \
  target/arm-unknown-optee/release/stripped_ta.<unique>

# 3. Sign (unique .ta.partial, then rename onto <uuid>.ta)
python3 /opt/optee/export-ta_arm32/scripts/sign_encrypt.py \
  --uuid <uuid-parsed-from-stripped-elf> \
  --key /opt/optee/export-ta_arm32/keys/default_ta.pem \
  --in target/arm-unknown-optee/release/stripped_ta.<unique> \
  --out target/arm-unknown-optee/release/<uuid>.ta.<unique>.partial
```

**Note:** `/tmp/cargo-optee-XXXXX` is a temporary directory containing the
embedded `arm-unknown-optee.json` target specification.

##### Example 3: Build aarch64 CA (Client Application)

**User Input:**
```bash
cargo-optee build ca \
  --optee-client-export /opt/optee/export-client \
  --manifest-path ./host/Cargo.toml
```

**cargo-optee translates to:**
```bash
OPTEE_CLIENT_EXPORT=/opt/optee/export-client \
cargo build --manifest-path ./host/Cargo.toml --target aarch64-unknown-linux-gnu --release \
  --config target.aarch64-unknown-linux-gnu.linker="aarch64-linux-gnu-gcc"

# 2. Strip
aarch64-linux-gnu-objcopy --strip-unneeded \
  target/aarch64-unknown-linux-gnu/release/<binary> \
  target/aarch64-unknown-linux-gnu/release/<binary>
```

#### Build Command Convention: Cargo to Low-Level Tools

This section explains how cargo orchestrates low-level tools to build the TA ELF
binary. We use an aarch64 no-std TA as an example.

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

Cargo then executes build scripts in dependency order to set up the build
directives:

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
   - Optional: `CARGO_PKG_VERSION`, `CARGO_PKG_DESCRIPTION` (for automatic TA
     config)
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
