---
permalink: /trustzone-sdk-docs/advanced-setup.md
---

# Advanced Setup: Customize Your Build Environment

## Platforms

To get started with Teaclave TrustZone SDK, you could choose either [QEMU for
Armv8-A](#develop-with-qemuv8) (QEMUv8) or [other
platforms](#develop-on-other-platforms) ([platforms OP-TEE
supported](https://optee.readthedocs.io/en/latest/general/platforms.html)) as
your development environment.

### Develop with QEMUv8

The OP-TEE libraries are needed when building Rust applications, so you should
finish the [Quick start with the OP-TEE Repo for
QEMUv8](#quick-start-with-the-op-tee-repo-for-qemuv8) part first. Then
initialize the building environment in Teaclave TrustZone SDK, build Rust
applications and copy them into the target's filesystem.

Teaclave TrustZone SDK is located in `[YOUR_OPTEE_DIR]/optee_rust/`. Teaclave
TrustZone SDK in OP-TEE repo is pinned to the release version. Alternatively,
you can try the develop version using `git pull`:

```sh
cd [YOUR_OPTEE_DIR]/optee_rust/
git pull github master
```

### Develop on Other Platforms

If you are building trusted applications for other platforms ([platforms OP-TEE
supported](https://optee.readthedocs.io/en/latest/general/platforms.html)). QEMU
and the filesystem in the OP-TEE repo are not needed.  You can follow these
steps to clone the project and build applications independently from the
complete OP-TEE repo. In this case, the necessary OP-TEE libraries are
initialized in the setup process.

1. The complete list of prerequisites can be found here: [OP-TEE
Prerequisites](https://optee.readthedocs.io/en/latest/building/prerequisites.html).

   ```sh
   # install dependencies
   sudo apt-get install android-tools-adb android-tools-fastboot autoconf \
   automake bc bison build-essential ccache cscope curl device-tree-compiler \
   expect flex ftp-upload gdisk iasl libattr1-dev libc6:i386 libcap-dev \
   libfdt-dev libftdi-dev libglib2.0-dev libhidapi-dev libncurses5-dev \
   libpixman-1-dev libssl-dev libstdc++6:i386 libtool libz1:i386 make \
   mtools netcat python-crypto python3-crypto python-pyelftools \
   python3-pycryptodome python3-pyelftools python-serial python3-serial \
   rsync unzip uuid-dev xdg-utils xterm xz-utils zlib1g-dev
   ```

   Alternatively, you can use a docker container built with our
   [Dockerfile](Dockerfile).

2. After installing dependencies or building the Docker image, fetch the source
   code from the official GitHub repository:
   
   ```sh
   git clone https://github.com/apache/teaclave-trustzone-sdk.git
   cd teaclave-trustzone-sdk
   ```

## Setup Building Environment

Currently, we support building on both `aarch64` and `x86_64` host machines, and
 they share the same steps.

1. Install the Rust environment and toolchains:

   ```sh
   ./setup.sh
   ```

2. Build OP-TEE libraries

   By default, the `OPTEE_DIR` is 
   `teaclave-trustzone-sdk/optee/`. OP-TEE submodules 
   (`optee_os` and `optee_client` for QEMUv8) will be initialized 
   automatically by executing:

   ```sh
   ./build_optee_libraries.sh optee/
   ```

3. Before building applications, set up the configuration:

   a. By default, the target platform is `aarch64` for both CA and TA. If 
   you want to build for the `arm` target, you can set up `ARCH`:

   ```sh
   export ARCH_HOST=arm
   export ARCH_TA=arm
   ```

   b. By default, the build is for `no-std` TA. If you want to enable 
   `std` TA, set the `STD` variable:

   ```sh
   export STD=y
   ```

4. Run this script to set up all toolchain and library paths:

   ```sh
   source environment
   ```

## Build Examples

Run this command to build all Rust examples:

``` sh
make examples
```

Or build your own CA and TA:

```sh
make -C examples/[YOUR_APPLICATION]
```

Besides, you can collect all example CAs and TAs to
`/teaclave-trustzone-sdk/out`:

```sh
make examples-install
```

## Run Rust Applications

Considering the platform has been chosen
([QEMUv8](#run-rust-applications-in-qemuv8) or
[other](#run-rust-applications-on-other-platforms)), the ways to run the Rust
applications are different.

### Run Rust Applications in QEMUv8

1. The shared folder is needed to share CAs and TAs with the QEMU guest system.
Recompile QEMU in OP-TEE to enable QEMU VirtFS:

```sh
(cd $OPTEE_DIR/build && make QEMU_VIRTFS_ENABLE=y qemu)
```

2. Copy all the Rust examples or your own applications to the shared folder:

```sh
mkdir shared_folder
cd [YOUR_OPTEE_DIR]/optee_rust/ && make examples-install)
cp -r [YOUR_OPTEE_DIR]/optee_rust/out/* shared_folder/
```

3. Run QEMU:

```sh
(cd $OPTEE_DIR/build && make run-only QEMU_VIRTFS_ENABLE=y
QEMU_VIRTFS_HOST_DIR=$(pwd)/shared_folder)
```

4. After the QEMU has been booted, you need to mount the shared folder in the
QEMU guest system (username: root), in order to access the compiled CA/TA from
QEMU. Run the command as follows in the QEMU guest terminal:

```sh
mkdir shared && mount -t 9p -o trans=virtio host shared
```

5. Then run CA and TA as [this
documentation](https://optee.readthedocs.io/en/latest/building/optee_with_rust.html)
describes.

### Run Rust Applications on Other Platforms

Copy the applications to your platform and run. 
