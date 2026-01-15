---
permalink: /trustzone-sdk-docs/emulate-and-dev-in-docker.md
---

# 🚀 Quick Start For QEMU Emulation

This guide walks you through building and running QEMU emulation using the
Teaclave TrustZone SDK.

We provide a Docker image with prebuilt QEMU and OP-TEE images to streamline the
entire Trusted Application (TA) development workflow. The image allows
developers to build TAs and emulate a guest virtual machine (VM) that includes
both the Normal World and Secure World environments.

## 1. Pull Development Docker Image

**Terminal A** (Main development terminal):
```bash
# Pull the pre-built development environment
$ docker pull teaclave/teaclave-trustzone-emulator-nostd-expand-memory:latest

# Clone the repository
$ git clone https://github.com/apache/teaclave-trustzone-sdk.git && \
  cd teaclave-trustzone-sdk

# Launch the development container
$ docker run -it --rm \
  --name teaclave_dev_env \
  -v $(pwd):/root/teaclave_sdk_src \
  -w /root/teaclave_sdk_src \
  teaclave/teaclave-trustzone-emulator-nostd-expand-memory:latest
```

## 2. Build the Hello World Example

Update: cargo-optee is available as an alternative to the original Makefile
system. See the [cargo-optee documentation](../cargo-optee/README.md) for
details.

### If you use Makefile:

**Still in Terminal A** (inside the Docker container):
```bash
# Build the Hello World example (both CA and TA)
make -C examples/hello_world-rs/
```
Under the hood, the Makefile builds both the Trusted Application (TA) and the
Host Application separately. After a successful build, you'll find the resulting
binaries in the `hello_world-rs` directory:
```bash
TA=ta/target/aarch64-unknown-linux-gnu/release/133af0ca-bdab-11eb-9130-43bf7873bf67.ta
HOST_APP=host/target/aarch64-unknown-linux-gnu/release/hello_world-rs
```

### If you use cargo-optee:

You can see the [cargo-optee build
commands](../cargo-optee/README.md#build-commands) for details.

## 3. Make the Artifacts Accessible to the Emulator
After building the Hello World example, the next step is to make the compiled
artifacts accessible to the emulator.

There are **two approaches** to do this. You can choose either based on your
preference:
- 📦 **Manual sync**: Explicitly sync host and TA binaries to the emulator
- ⚙️ **Makefile integration**: Use `make emulate` to build and sync in one step
  (only when you use Makefile for building)

#### Option 1: Manual Sync via `sync_to_emulator`
We provide a helper command called `sync_to_emulator`, which simplifies the
process of syncing the build outputs to the emulation environment. Run the
following commands inside the container:
```bash
sync_to_emulator --ta $TA
sync_to_emulator --host $HOST_APP
```
Run `sync_to_emulator -h` for more usage options.

#### Option 2: Integrate sync with TA's Makefile
For convenience during daily development, the sync invocation can be integrated
into the Makefile. In the `hello_world-rs` example, an `emulate` target is
provided. This helps automatically build the artifacts and sync them to the
emulator in one step:
```bash
make -C examples/hello_world-rs/ emulate
```

## 4. Multi-Terminal Execution

The emulation workflow requires three additional terminals to monitor various
aspects of the system:

- **Terminal B**: 🖥️ **Normal World Listener** - Provides access to the guest
  VM shell
- **Terminal C**: 🔒 **Secure World Listener** - Monitors Trusted Application
  output logs  
- **Terminal D**: 🚀 **QEMU Control** - Controls the QEMU emulator

Built-in commands are provided in the Docker image. These commands are located
in `/opt/teaclave/bin/` and are included in the default user's $PATH.

You may use `bash -l` or the full path when executing with docker exec.

**Terminal B** (Guest VM Shell):
```bash
# Connect to the guest VM shell for running commands inside the emulated environment
$ docker exec -it teaclave_dev_env bash -l -c listen_on_guest_vm_shell

# Alternative: Use full path
$ docker exec -it teaclave_dev_env /opt/teaclave/bin/listen_on_guest_vm_shell
```

**Terminal C** (Secure World Output Monitor):
```bash
# Monitor Trusted Application output logs in real-time
$ docker exec -it teaclave_dev_env bash -l -c listen_on_secure_world_log

# Alternative: Use full path  
$ docker exec -it teaclave_dev_env /opt/teaclave/bin/listen_on_secure_world_log
```

## 5. Start the Emulation

After the listeners are set up, we can start the QEMU emulator.

**Terminal D** (QEMU Control):
```bash
# Launch QEMU emulator with debug output and connect to monitoring ports
$ docker exec -it teaclave_dev_env bash -l -c "LISTEN_MODE=ON start_qemuv8"
```

> ⏳ **Wait for the QEMU environment to fully boot...** You should see boot
messages in Terminal D and the guest VM shell prompt in Terminal B.

After QEMU in Terminal D successfully launches, switch to Terminal B, which
provides shell access to the guest VM's normal world.

**Terminal B** (Inside Guest VM): From this shell, you'll find that the
artifacts synced in **Step 3** are already available in the current working
directory. Additionally, the `ta/` and `plugin/` subdirectories are
automatically mounted to be used by TEE OS during TA execution and plugin
loading.

For more details on the mount configuration, refer to the
`listen_on_guest_vm_shell` command in the development environment.

```bash
# tree
.
|-- host
|   `-- hello_world-rs
|-- plugin
`-- ta
    `-- 133af0ca-bdab-11eb-9130-43bf7873bf67.ta

3 directories, 2 files
```
This makes it especially convenient for iterative development and frequent code
updates.

Now we are ready to interact with the TA from normal world shell.
```bash
# Execute the Hello World Client Application
$ ./host/hello_world-rs
```
The secure world logs, including TA debug messages, are displayed in **Terminal
C**.

## 6. Iterative Development with Frequent Code Updates and Execution
During active development and debugging, you can leave Terminals B, C, and D
open to avoid restarting them each time. Simply return to Terminal A, and repeat
Step 2 (build) and Step 3 (sync) to rebuild and update the artifacts. Once
synced, switch to Terminal B to re-run the client application. This setup
streamlines iterative development and testing.

## Summary
By following this guide, you can emulate and debug Trusted Applications using
our pre-configured Docker-based development environment.  

- **Terminal A** serves as the main interface for building and syncing
  artifacts. 
- **Terminal B** gives access to the normal world inside the guest VM, where you
can run client applications like the Hello World example. 
- **Terminal C** captures logs and debug output from the secure world, making it
easy to trace TA behavior. 
- **Terminal D** controls the QEMU emulator and shows system-level logs during
boot and runtime. 

Together, these terminals provide a complete and efficient workflow for
TrustZone development and emulation.

### Development Environment Details
The setup scripts and built-in commands can be found in `/opt/teaclave/`. Please
refer to the Dockerfile in the SDK source repository for more information about
how we set up the development environment.