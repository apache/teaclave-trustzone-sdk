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
OP-TEE configuration provides. Both the AArch64 and AArch32 images use these
patches together with the expanded TZDRAM make arguments.

Relevant Patch File:

1. qemu-qemu_expand_secure_memory.patch: Increases the size of VIRT_SECURE_MEM
    in QEMU.
2. arm-atf_expand_secure_memory.patch: Updates ARM Trusted Firmware definitions
    to match the QEMU memory expansion.

## Patch for the user VA range overflow

On a 32-bit core with LPAE, OP-TEE can choose the top 1 GB slot for user
mappings. That range ends at 4 GB, which does not fit in a 32-bit `vaddr_t`,
so the range end wrapped to 0 and no user TA could be loaded. The patch caps
the range end at the last `CORE_MMU_PGDIR_SIZE` boundary, which leaves the
last 2 MB of the slot unused but keeps the slot usable.

Relevant Patch File:

1. optee-os_user_va_range_overflow.patch: Caps the user VA range end in
    `core/mm/vm.c`.

## Patches for enabling ssh

Our CI workflow automates command execution via SSH and handles CA/TA deployment
using SCP, providing a seamless local-like interface for the QEMU image.

Relevant Patch File:

1. optee-build_enable_ssh.patch: update build configurations to enable the ssh
    server.
