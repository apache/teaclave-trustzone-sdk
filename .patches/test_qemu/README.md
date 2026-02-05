# Patches to Customize Our Test QEMU Image

This folder contains patch files used to build our custom QEMU test image.

## Patches for disabling watch dog test

The watchdog test significantly impacts OP-TEE OS and tee-supplicant
initialization performance, particularly on AMD64 hosts. Disabling it can
improve boot time and runtime efficiency.

Relevant Patch File:

1. optee-build_disable_wd_test.patch: Updates build configurations to disable
    the watch dog test.

## Patches for Expand Memory

Some of our tests require more Trusted Application (TA) memory than the default
OP-TEE configuration provides.

Relevant Patch File:

1. qemu-qemu_expand_secure_memory.patch: Increases the size of VIRT_SECURE_MEM
    in QEMU.
2. arm-atf_expand_secure_memory.patch: Updates ARM Trusted Firmware definitions
    to match the QEMU memory expansion.
3. optee-build_expand_memory.patch: Updates build configurations to reflect the
    expanded memory setup.

## Patches for enabling ssh

Our CI workflow automates command execution via SSH and handles CA/TA deployment
using SCP, providing a seamless local-like interface for the QEMU image.

Relevant Patch File:

1. optee-build_enable_ssh.patch: update build configurations to enable the ssh
    server.
