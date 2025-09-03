---
permalink: /trustzone-sdk-docs/writing-rust-tas-using-optee-utee-build.md
---

# Writing Rust TAs Using optee-utee-build

Currently we provide a `optee-utee-build` crate to simplify the compilcated
building process of TA, and we recommend everyone use it in future developement.

* For legacy app structures migrating to use this crate, refer to [Migration
  Guide](#migration-guide)
* If you're new to development, start with [Minimal Example](#minimal-example)
* To customize the build process, see [Customization](#customization)

# Minimal Example

Assuming currently we are developing a `hello_world` TA, and we want to build it
with `optee-utee-build` crate, we can do it by following steps.

Firstly, we should add `optee-utee-build` in `build-dependencies`:

```shell
cargo add --build optee-utee-build
```

Secondly, we set a `ta_config` and call `optee-utee-build::build` with it in
build.rs:

```rust
use proto;
use optee_utee_build::{TaConfig, Error, RustEdition};

fn main() -> Result<(), Error> {
    let ta_config = TaConfig::new_default_with_cargo_env(proto::UUID)?;
    optee_utee_build::build(RustEdition::Before2024, ta_config)
}
```

It will generate a `user_ta_header.rs` file and setup all the required
configurations of the linker of rustc.

Finally, we include the generated `user_ta_header.rs` in the source codes,
normally we put it in `src/main.rs`.

```rust
// src/main.rs
include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
```

After that, everything finished, we can start building the TA now.

For full codes, you can check the [`hello_world-rs example`](https://github.com/apache/teaclave-trustzone-sdk/tree/main/examples/hello_world-rs/ta)

## Explaination of Minimal Example

### 1. The TaConfig

This is a struct that use for the configuration of the TA we are developing, it
has some public fields:

1. **uuid**: the identifier of TA.
2. **ta_flags**: combination of some bitflags.  
   for available values, you may check [user_ta_header.h in optee_os](https://github.com/OP-TEE/optee_os/blob/c2e42a8f03a5bb6b894ef85ae409f54760c1f50e/lib/libutee/include/user_ta_header.h#L13-L53)
3. **ta_data_size**: the size in bytes of the TA allocation pool.
4. **ta_stack_size**: the size in bytes of the stack used for TA execution.
5. **ta_version**: a version string of TA, should be in semver format.
6. **ta_description**: the desciption of TA.
7. **trace_level**: the default trace level of TA.  
   for available values, you may check [trace_levels.h in optee_os](https://github.com/OP-TEE/optee_os/blob/c2e42a8f03a5bb6b894ef85ae409f54760c1f50e/lib/libutils/ext/include/trace_levels.h#L26-L31)
8. **trace_ext**: an extra prefix string when output trace log.
9. **ta_framework_stack_size**: the size in bytes of the stack used for Trusted
Core Framework.  
   currently used for trace framework and invoke command, should not be less
   than 2048.
10. **ext_properties**: the extra custom properties.

We can construct the `TaConfig` by providing all of the public fields manually,
or use our standard constructor:

1. **new_default**: construct a default TaConfig by providing uuid, ta_version 
and ta_description, with other configurations set to suggested values, you can 
update those configurations later.
2. **new_default_with_cargo_env**: it's a constructor wrapped with new_default, 
but take `version` and `description` from cargo.toml so simply providing a uuid 
as parameter is enough.

### 2. The RustEdition

The generated `user_ta_header.rs` must be different between `edition of 2024`
and `edition before 2024`, and currently there is no official stable way to know
what edition we are compiling with, so we provide a argument to set with.

> #### What’s the difference?
> the generated `user_ta_header.rs` file include some const variables and global
functions tagged with `no_mangle` and `link_section`, start from rust edition of
2024, they must be wrapped with unsafe, or rustc will output a compilation error
(while before edition of 2024 it must not, or rustc will output a syntax error).

# Customization

`optee-utee-build` provide some structs for flexible use.

### 1. Builder

Instead of calling the `build` function directly, you can use Builder for
customization.

Usage:

```Rust
use proto;
use optee_utee_build::{TaConfig, Builder, Error, RustEdition, LinkType};

fn main() -> Result<(), Error> {
    let ta_config = TaConfig::new_default_with_cargo_env(proto::UUID)?;
    Builder::new(RustEdition::Before2024, ta_config)
      .out_dir("/tmp")
      .header_file_name("my_generated_user_ta_header.rs")
      .link_type(LinkType::CC)
      .build()
}
```

As you can see from the codes, there are some customizations of the builder:
1. **out_dir**: change directory of output files.  
   default to OUT_DIR by cargo.
2. **header_file_name**: change name of output header file.  
   default to `user_ta_header.rs`
3. **link_type**: set link_type manually.  
   there are some difference in parameters in
   linkers between `CC` and `LD` types, for example, `--sort-section` in `CC` types
   of linkers changes to `-Wl,--sort-section`, we will try to detect current linker
   that cargo using, you can use this function to set it manually if you think our
   detection mismatch.

### 2. Linker
For developers who prefer to use a hand-written `user_ta_header.rs` and only
want `optee-utee-build` to handle the linking process, they can use the
`Linker`, otherwise, try `Builder` instead.

Usage:

``` rust
use optee_utee_build::{Linker, Error};
use std::env;

fn main() -> Result<(), Error> {
  let out_dir = env::var("OUT_DIR")?;
  Linker::auto().link_all(out_dir)?;
  Ok(())
}
```

When linking manually, developers construct a `Linker` and calling the
`link_all` method by providing the out_dir, and linker will generate some
required files (link script, etc, used by linker) into out_dir and handle all
the linking stuff.

In above codes, we use `auto` to construct the linker, it will detect current
linker that cargo using automatically, you can use `new` function to construct
the linker manually if you think our detection mismatch.
```rust
use optee_utee_build::{Linker, Error, LinkType};
use std::env;

fn main() -> Result<(), Error> {
  let out_dir = env::var("OUT_DIR")?;
  Linker::new(LinkerType::CC).link_all(out_dir)?;
  Ok(())
}
```

### 3. HeaderFileGenerator
For developers who prefer to do the linking themselves and only want
`optee-utee-build` to generate the header file, they can use the
`HeaderFileGenerator`, otherwise, try `Builder` instead.

Usage:

```rust
use optee_utee_build::{HeaderFileGenerator, TaConfig, RustEdition, Error};

fn main() -> Result<(), Error> {
  const UUID: &str = "26509cec-4a2b-4935-87ab-762d89fbf0b0";
  let ta_config = TaConfig::new_default(UUID, "0.1.0", "example")?;
  let codes = HeaderFileGenerator::new(RustEdition::Before2024).generate(&ta_config)?;
  Ok(std::io::Write("/tmp/user_ta_header.rs", codes.as_bytes())?)
}

```

# Migration Guide

For developers still using `const configuration values` in `src/main.rs` and
`custom build scripts` in `build.rs`(described in [\[migrating-to-new-building-env\]](https://github.com/apache/teaclave-trustzone-sdk/blob/main/docs/migrating-to-new-building-env.md)),
they can upgrade to `optee-utee-build` by following step:

Firstly, add `optee-utee-build` as `build-dependencies`:

```shell
cargo add --build optee-utee-build
```

Secondly, in `build.rs`, remove codes of `custom build scripts`, and use
`optee_utee_build::build` instead:

```rust
// ... other imports
use optee_utee_build::{TaConfig, Error}

fn main() -> Result<(), Error> {
  // should customize the ta_config with the same as const configuration values
  // in your src/main.rs
  let ta_config = TaConfig::new_default_with_cargo_env(proto::UUID)?
    .ta_stack_size(10 * 1024); 
  optee_utee_build::build(RustEdition::Before2024, ta_config)?;

  // ... other build scripts
}
```

Thirdly, remove `const configuration values` in `src/main.rs`, keep the line of
`include user_ta_header.rs`.

```rust
/// ... other codes in src/main.rs

/* remove const configuration values, move them to TaConfig in src/main.rs
// TA configurations
const TA_FLAGS: u32 = 0;
const TA_DATA_SIZE: u32 = 32 * 1024;
const TA_STACK_SIZE: u32 = 2 * 1024;
const TA_VERSION: &[u8] = b"0.1\0";
const TA_DESCRIPTION: &[u8] = b"This is a hello world example.\0";
const EXT_PROP_VALUE_1: &[u8] = b"Hello World TA\0";
const EXT_PROP_VALUE_2: u32 = 0x0010;
const TRACE_LEVEL: i32 = 4;
const TRACE_EXT_PREFIX: &[u8] = b"TA\0";
const TA_FRAMEWORK_STACK_SIZE: u32 = 2048;
*/

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));  // keep this line
```

Finally, delete the useless `ta_static.rs` and start building now.
