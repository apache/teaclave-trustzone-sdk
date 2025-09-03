---
permalink: /trustzone-sdk-docs/building-rust-ca-as-android-elf.md
---

# Building Rust CA as Android ELF

In Teaclave TrustZone SDK, example CAs are built as ARM64 Linux ELF by default.
Besides, you can follow these steps to build Rust CAs running on the Android
platform:

1. Download Android NDK toolchain

```
$ wget https://dl.google.com/android/repository/android-ndk-r21e-linux-x86_64.zip
$ unzip android-ndk-r21e-linux-x86_64.zip
```

2. Add the android target

```
$ rustup target add aarch64-linux-android
```

3. Set toolchains for the target. Add PATH env:

```
export PATH=$PATH:/your/path/to/android-ndk-r21e/toolchains/llvm/prebuilt/linux-x86_64/bin/
```

4. Edit `teaclave-trustzone-sdk/.cargo/config`, add:

```
[target.aarch64-linux-android]
linker = "aarch64-linux-android28-clang"
ar = "aarch64-linux-android-ar"
```

5. Copy Android libteec.so to
`/teaclave-trustzone-sdk/optee/optee_client/out/export/usr/lib`. 

- Note: If you've not built the libteec.so of Android, you can build it using:
```
$ cd /path/to/optee/optee_client/
$ ndk-build APP_BUILD_SCRIPT=./Android.mk NDK_PROJECT_PATH=. NDK_LOG=1 APP_PLATFORM=android-29
```

6. Modify CA's Makefile:

```
NAME := hello_world-rs
TARGET := aarch64-linux-android
OPTEE_DIR ?= ../../../optee
OUT_DIR := $(CURDIR)/target/$(TARGET)/release

all: host

host:
        @cargo build --target $(TARGET) --release --verbose
clean:
        @cargo clean
```

7. build:

```
$ make -C examples/hello_world-rs/host
```
