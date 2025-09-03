---
permalink: /trustzone-sdk-docs/migrating-to-new-building-env.md
---

# Migrating To New Building ENV

> After optee-utee-build release, this doc is keeping for developers 
who intend to know the detail of building process, we suggest use 
[optee-utee-build](./writing-rust-tas-using-optee-utee-build.md) for building 
instead.

## Migration Guide: Moving from `master` to `main` Branch (Post-Oct 2024)

Since the `main` branch (after October 2024) introduces breaking changes 
to the build environment, if users of the legacy `master` branch want to 
keep upstream or use a new version of the Rust toolchain, they will need 
to migrate their TA to the new environment.

Note that the migration is mainly for building scripts to support both 
`no-std` and `std` building for TA, no need for modifying your application 
code.

### Current Structure

We have retained almost the same structure as the original but removed 
`ta_arm.lds` and `ta_aarch64.lds` from the directory structure. Besides 
we have some modification on `ta/ta_static.rs`, `ta/build.rs` and all 
`Makefile`s. (See the explanation in next part). 

For example the current `examples/acipher-rs/`:
```
examples/acipher-rs/
├── host
│   ├── Cargo.toml
│   ├── Makefile
│   └── src
│       └── main.rs
├── Makefile
├── proto
│   ├── build.rs
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── ta
│   ├── build.rs
│   ├── Cargo.toml
│   ├── Makefile
│   ├── src
│   │   └── main.rs
│   ├── ta_static.rs
│   └── Xargo.toml
└── uuid.txt
```


### Changes in Build Scripts  

1. **TA linking script**: `ta_arm.lds` and `ta_aarch64.lds`.  
   These linking scripts define the low-level TA ELF sections arrangement 
   (e.g., `.text` section in ELF). They have been removed, and we now use 
   the `lds` file from OP-TEE's TA dev-kit, for example, located in 
   `optee_os/out/arm-plat-vexpress/export-ta_arm64/src/ta.ld.S`. This 
   change helps to stay upstream with OP-TEE OS and makes it more stable 
   when running on OP-TEE OS.

2. **`ta_static.rs`**: C FFI primitives, such as `ta_heap_size` and `ta_props`.  
   This file helps to set TA properties in a C-like manner in the TA ELF 
   for OP-TEE OS to load.  
   The change involves modifying imports of primitives, e.g., from 
   `libc::c_int` to `core::c_int`, and from `std::u64::MAX` to 
   `core::primitive::u64::MAX`. This helps ensure support for both `no_std` 
   and `std`-based environments.

3. **`build.rs`**:   
   Since TA is not a normal ELF, it has a header before the ELF sections.  
   This file is the main entry point for building a TA as ELF and adding 
   the specific header. It uses configurations such as `ta_static.rs`, 
   `user_ta_header.rs`, and the linking script `ta.ld.S`. It also defines 
   the linking with OP-TEE's C libraries (`libutee` and `libutils`) from 
   OP-TEE's TA dev-kit.
   
   The changes are:
   
   a. Move linking parameters from the original 
   [`/.cargo/config`](https://github.com/apache/teaclave-trustzone-sdk/blob/master/.cargo/config):
	This change is primarily designed to accommodate more complex build targets.
	For standard TAs, the specific build targets are `aarch64-unknown-optee-trustzone` 
	and `arm-unknown-optee-trustzone`.
	For example, in the no-std mode for aarch64, both no-std TAs and CAs are built 
	with the `aarch64-unknown-linux-gnu` target. However, in std mode, TAs are 
	built with the `aarch64-unknown-optee-trustzone` target, while CAs remain 
	built with the `aarch64-unknown-linux-gnu` target.
	This change allows us to decouple TA's linking parameters from the target, as 
	they are now defined within TA's `build.rs`.

   b. Add `cargo:rustc-link-arg=--no-warn-mismatch` to work around 
   the EABI version mismatch linking error: symbols.o with EABI version 0 
   and other objects are EABI version 5.  

5. **ENV variables**:  
   The original script for setting the toolchain path has some modifications. 
   Due to the more complex building options mentioned above, `CROSS_COMPILE_{HOST, TA}` 
   and `TARGET_{HOST, TA}` should be set by `source environment`. 
   You should also set whether you want to build in `STD` mode (`export STD=y`) 
   and specify the target architecture (`ARM32` or `AArch64`) for both CA and TA. 
   Running `source environment` will set up all toolchains and libraries.

6. **Makefile Polishing**:  
   a. Top-level Makefile (`examples/*/Makefile`): Reads the `CROSS_COMPILE_{HOST, TA}` 
   and `TARGET_{HOST, TA}`.  
   b. `host/Makefile`: Simplified and polished for the changes in ENV variables.  
   c. `ta/Makefile`: For `std` TAs, checks if the `STD` environment variable is set, 
   and further simplifications and polish are done.

### Step 1: Migrating Projects

#### Case 1: Default Migration (No Custom Modifications to Build Scripts)
If you have developed based on one of our example structures and haven't 
modified the build scripts mentioned above, you can simply copy a current 
example and move your code into it.  
Note that the `Makefile` for `std` TAs has tiny differences from the `no_std` 
one. If you are using a `no_std` TA, refer to `hello_world-rs`. For `std` TAs, 
refer to `serde-rs`.

We provide a shell script to assist with this migration (you may need to make 
small adjustments based on whether you are building in `no_std` or `std` mode). 
Here is an example for `no_std`:

```bash
TARGET_EXAMPLE="your_project"
OLD_ROOT_PATH="/path/to/old/sdk"
NEW_PATH="/path/to/current/sdk"

# Duplicate the hello-world example in the new path as a template
cp -r ${NEW_PATH}/examples/hello_world-rs ${NEW_PATH}/examples/${TARGET_EXAMPLE}

# Remove the source code directory and copy from the old path to the new path
# including: src/ and Cargo.toml in host, ta, proto
(cd ${NEW_PATH}/examples/${TARGET_EXAMPLE}/host && rm -rf src/ Cargo.* && \
cp -r ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/host/src . && \
cp ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/host/Cargo.toml .)
(cd ${NEW_PATH}/examples/${TARGET_EXAMPLE}/ta && rm -rf src/ Cargo.* && \
cp -r ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/ta/src . && \
cp ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/ta/Cargo.toml .)
(cd ${NEW_PATH}/examples/${TARGET_EXAMPLE}/proto && rm -rf src/ Cargo.* && \
cp -r ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/proto/src . && \
cp ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/proto/Cargo.toml .)

# Copy the UUID file from the old path to the new path
cp ${OLD_ROOT_PATH}/examples/${TARGET_EXAMPLE}/uuid.txt \
${NEW_PATH}/examples/${TARGET_EXAMPLE}/uuid.txt

# Update binary names in host/Cargo.toml and host/Makefile
sed -i "s/hello_world-rs/${TARGET_EXAMPLE}/g" \
${NEW_PATH}/examples/${TARGET_EXAMPLE}/host/Cargo.toml
sed -i "s/hello_world-rs/${TARGET_EXAMPLE}/g" \
${NEW_PATH}/examples/${TARGET_EXAMPLE}/host/Makefile
```

#### Case 2: Custom Migration (With Modified Build Scripts)

If you have made changes to your build scripts, follow the steps below to 
manually migrate those files:

1. **TA linking script `ta_arm.lds` and `ta_aarch64.lds`**:  
	Usually, developers don't need to modify those files. If you have made any
	changes, compare the diff between your file and
	`optee_os/out/arm-plat-vexpress/export-ta_{arm64, arm32}/src/ta.ld.S` in the 
	current SDK.
  This `ta.ld.S` file is currently not included in SDK but in OPTEE_OS repo.

3. **`ta_static.rs`**:  
   Usually, developers don't need to modify this file. If you have made 
   modifications to this file, compare them with the latest version here:  
   [ta_static.rs diff](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..9e3906e9d82f0471e96bf892afe0df37dd90a86e#diff-c0cdd7b28f558bd417069b8e60ed35b70ac1cd01e68e3c0ba6c7311a5a444e22)

4. **`build.rs`**:  
   Usually, developers don't need to modify this file. If you have made
   changes to link other libraries or dependencies in `build.rs`, compare
   the two versions and migrate accordingly:  
   [build.rs diff](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..9e3906e9d82f0471e96bf892afe0df37dd90a86e#diff-c07432a8a8ecbc1f00799a2bd008bd8dcbba9d58fd0a9e5815b835e4ed425e86)

5. **Makefiles**:  
You may have modified some of the Makefiles. Please compare them 
with the current versions to ensure compatibility:

- **For `no_std` builds**:  
   - [Top-level Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-df315bfec3c0b8e84c64b31e4450660ea66c33aa833f5b1b9d76250481c15887)  
   - [Host Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-96468cc392cceb21806dbfb2dd24007d772f19992955ed81c4979a45f753378a)  
   - [TA Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-29c530c8f83308f34fae9b3516015f07fa80c1b879cc9a8834c4dfaa497af1a5)

- **For `std` builds**:  
   - [Top-level Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-15685120d44f0ca4ea11ac90799a621f19378cebf5b018792ebc25bee68c3824)  
   - [Host Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-dfb3cbc25e6b4bad652b716b9d051c9fb7c45d2d8303caa936666774c49a624a)  
   - [TA Makefile](https://github.com/apache/teaclave-trustzone-sdk/compare/cd19ac2e1c3cb1a848d5131d4af8138d84be8708..dc1523cbcf6c716213854d9a16d39b8d498a9bb6#diff-e0618a8a49e0ac65dd1acd48a0108c280a3821bcfb233f46f4baa56c77369001)

### Step 2: **Update `Cargo.toml`**  
You may need to update your `Cargo.toml` file to include newer 
versions of crates that depend on the new Rust toolchain. Refer to 
the `rust-toolchain.toml` file to verify the current toolchain. If 
you update any crates, be prepared for potential code changes to 
accommodate new interfaces.

### Step 3: **Build and Resolve Errors**  
After updating the necessary files, rebuild the project. During the 
process, errors might arise due to crate version mismatches or 
other updates. Make sure to resolve these errors by adjusting your 
code accordingly.
