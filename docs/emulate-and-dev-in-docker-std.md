---
permalink: /trustzone-sdk-docs/emulate-and-dev-in-docker-std.md
---

# 🚀 Developing TAs with Rust Standard Library in Docker

This guide covers the **dev-env with std support** that enables **developing TA
using Rust standard library (std)**, compared to the regular no-std environment
documented in [emulate-and-dev-in-docker.md](emulate-and-dev-in-docker.md).

The **dev-env with std support** provides a complete setup for building TAs that
can use Rust's standard library features like collections, networking, etc.

> 📖 **Prerequisites**: Read the [original Docker development
> guide](emulate-and-dev-in-docker.md) first. This document focuses only on
> std-specific differences and capabilities.

## What the Dev-Env with Std Support Provides

The **dev-env with std support** enables **developing TA using Rust std** by
providing:
- **Flexible configuration management** - Switch between std/no-std modes and
  architectures dynamically
- **Rust standard library tailored for OP-TEE** - Build TAs using collections,
  networking, serialization capabilities
- **Mixed development support** - Combine different host and TA architectures,
  including switching between no-std/std in the same project

## 1. Setting Up the Dev-Env with Std Support

### Pull the Docker Image
```bash
# Pull the dev-env with std support for developing TA using Rust std
$ docker pull teaclave/teaclave-trustzone-emulator-std-expand-memory:latest

# Launch the dev-env container
$ docker run -it --rm \
  --name teaclave_dev_env \
  -v $(pwd):/root/teaclave_sdk_src \
  -w /root/teaclave_sdk_src \
  teaclave/teaclave-trustzone-emulator-std-expand-memory:latest
```

### One-Time Setup Inside Container
```bash
# Create symbolic link to make it compatiable with existing SDK examples
$ ln -s $RUST_STD_DIR rust
```

> 📝 **Note**: This symlink is required for current SDK examples due to
> hardcoded std dependency paths in Cargo.toml. Your own projects may organize
> std files differently.

## 2. Configuration Management System

The key difference is the **unified configuration system** that allows switching
between std/no-std modes and different architectures on demand.

And [cargo-optee](../cargo-optee/README.md#configuration-system) is available as
an alternative to the original configuration management tool: switch_config.

### If you use switch_config:

#### Check Available Configurations
```bash
# Show current active configuration
$ switch_config --status

# List all supported configurations
$ switch_config --list
```

**TA Configurations Available:**
- `std/aarch64`, `std/arm32` - With Rust standard library
- `no-std/aarch64`, `no-std/arm32` - Without standard library

**Host Configurations Available:** `aarch64`, `arm32`

**Default Configuration:** Host=`aarch64`, TA=`std/aarch64`

#### Switching Between Configurations
```bash
# Switch TA configurations
$ switch_config --ta std/aarch64     # Enable std for 64-bit TA
$ switch_config --ta std/arm32       # Enable std for 32-bit TA  
$ switch_config --ta no-std/aarch64  # Disable std, use 64-bit no-std

# Switch host architecture
$ switch_config --host arm32         # Use 32-bit host

# Mixed development example: 32-bit host + 64-bit std TA
$ switch_config --host arm32 && switch_config --ta std/aarch64
```

### If you use cargo-optee:

You can see the [cargo-optee configuration
system](../cargo-optee/README.md#configuration-system) for details.

## 3. Building and Target Differences

Follow the [original building
instructions](emulate-and-dev-in-docker.md#2-build-the-hello-world-example), but
note these important target differences:

| Configuration | TA Target | Build Tool | Host Target |
|---------------|-----------|------------|-------------|
| `std/*` | `*-unknown-optee` | `xargo` | `*-unknown-linux-gnu` |
| `no-std/*` | `*-unknown-linux-gnu` | `cargo` | `*-unknown-linux-gnu` |

**Example std build output:**
```bash
TA=ta/target/aarch64-unknown-optee/release/133af0ca-bdab-11eb-9130-43bf7873bf67.ta
```

## 4. Hello World Example: Std vs No-Std

### Build with Default Std Configuration
```bash
# Build hello world with std/aarch64 (default configuration)
$ cd examples/hello_world-rs/
$ make
```

**Result:** TA built with std enabled, targeting `aarch64-unknown-optee`:
```bash
TA=ta/target/aarch64-unknown-optee/release/133af0ca-bdab-11eb-9130-43bf7873bf67.ta
```

### Switch to No-Std and Rebuild
```bash
# Switch TA to no-std mode and rebuild
$ switch_config --ta no-std/aarch64
$ make clean && make
```

**Result:** TA now targets `aarch64-unknown-linux-gnu` (no-std):
```bash
TA=ta/target/aarch64-unknown-linux-gnu/release/133af0ca-bdab-11eb-9130-43bf7873bf67.ta
```

If you are using cargo-optee, the relevant workflow is already clearly
documented in the [cargo-optee appendix](../cargo-optee/README.md#appendix), so
it will not be repeated here.

## 5. Emulation and Execution

The emulation process is identical to the no-std environment. Follow [sections
3-6 of the original
guide](emulate-and-dev-in-docker.md#3-make-the-artifacts-accessible-to-the-emulator)
for complete emulation setup instructions.