---
permalink: /trustzone-sdk-docs/expanding-ta-secure-memory-on-qemuv8.md
---

# Expanding TA Secure Memory on QEMUv8

Since some Rust examples such as `tls_server-rs` and `tls_client-rs`  require
larger TA memory (about 18M heap), we've expanded TA secure memory on OP-TEE
QEMUv8 platform. On QEMUv8 platform it supports 7M TA memory originally, after
expanding it supports 27M TA memory at most.

We modified the firmware and configuration of QEMU, ATF and OPTEE. You can
download the pre-built image from
https://nightlies.apache.org/teaclave/teaclave-trustzone-sdk/ or patch the code
and build the images by yourself.

For details on the modifications, please refer to the 
[Patches](https://github.com/apache/teaclave-trustzone-sdk/.patches/test_qemu)

Finally, build images:

```
$ cd optee-repo/build
$ make
```
