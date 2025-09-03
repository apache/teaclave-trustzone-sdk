# Apache Teaclave™ TrustZone SDK

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/tag/apache/teaclave-trustzone-sdk?label=release&sort=semver)](https://github.com/apache/teaclave-trustzone-sdk/releases)
[![Homepage](https://img.shields.io/badge/site-homepage-blue)](https://teaclave.apache.org/)

Apache Teaclave™ TrustZone SDK (Rust OP-TEE TrustZone SDK) provides abilities to build
safe TrustZone applications in Rust. The SDK is based on the
[OP-TEE](https://www.op-tee.org/) project which follows
[GlobalPlatform](https://globalplatform.org/) [TEE
specifications](https://globalplatform.org/specs-library/tee-internal-core-api-specification/)
and provides ergonomic APIs. In addition, it enables the capability to write
TrustZone applications with Rust's standard library (std) and many third-party
libraries (i.e., crates). 

Apache Teaclave™ TrustZone SDK provides two development modes for Rust TAs: `no-std`
and `std`.  We recommend using `no-std` by default. For a detailed comparison, please refer
to [Comparison](docs/ta-development-modes.md).

**UPDATES:** We have developed a new build environment on the `main` branch, 
which will now be the only branch for development and maintenance and includes 
breaking changes to the legacy `master` branch.
If you're using the `master` branch and wish to migrate to the new development 
branch (`main`), please refer to the 
[migration guide](docs/migrating-to-new-building-env.md).

## 🚀 Quick & Easy Start: Hello World TA in Emulator

Developing Trusted Applications (TAs) often requires specific hardware, which 
can be a barrier for many developers. To address this, we provide a prebuilt 
Docker environment that allows you to experience TAs without the need for 
physical hardware.

The Docker image automates the entire setup process for TrustZone emulation 
in QEMU, enabling you to focus on writing and testing your applications 
efficiently, without the hassle of manual configuration.

**Choose your development mode in Emulator:**
- 🚀 [Quick Emulation And Development in Docker](docs/emulate-and-dev-in-docker.md) 
- 🚀 [Developing TAs with Rust Standard Library](docs/emulate-and-dev-in-docker-std.md)

## Advanced Setup: Customize Your Build Environment

In addition to developing and testing Trusted Applications (TAs) in the QEMU 
emulator, setting up build configurations for specific hardware targets are 
also necessary.  For detailed instructions on customizing your build environment, 
please refer to the [Advanced Setup Documentation](docs/advanced-setup.md).

For other tips regarding the support Rust Examples, TA debugging, expanding 
secure memory, please refer to the [docs/ directory](docs/README.md).

## Publication

More details about the design and implementation can be found in our paper
published in ACSAC 2020:
[RusTEE: Developing Memory-Safe ARM TrustZone
Applications](https://csis.gmu.edu/ksun/publications/ACSAC20_RusTEE_2020.pdf).
Here is the BiBTeX record for your reference.

```bibtex
@inproceedings{wan20rustee,
    author    = "Shengye Wan and Mingshen Sun and Kun Sun and Ning Zhang and Xu
He",
    title     = "{RusTEE: Developing Memory-Safe ARM TrustZone Applications}",
    booktitle = "Proceedings of the 36th Annual Computer Security Applications
Conference",
    series    = "ACSAC '20",
    year      = "2020",
    month     = "12",
}
```

## Contributing

Apache Teaclave™ is open source in [The Apache
Way](https://www.apache.org/theapacheway/),
we aim to create a project that is maintained and owned by the community. All
kinds of contributions are welcome.
Thanks to our [contributors](https://teaclave.apache.org/contributors/).

Apache Teaclave™ follows the Apache Software Foundation (ASF) model, which does not 
require `Signed-off-by` or other commit trailers. While such tags 
(e.g., DCO-style trailers like `Signed-off-by`, `Reviewed-by`) are welcome, 
they are optional and not enforced. Pull requests with or without them are 
equally welcome.

However, DCO-style tags cannot substitute for the Contributor License 
Agreement (CLA). Major contributions and all committers must have a signed 
CLA on file, as [required by the ASF](https://www.apache.org/licenses/contributor-agreements.html#clas).


## Community

- Join us on our [mailing
  list](https://lists.apache.org/list.html?dev@teaclave.apache.org).
- Follow us at [@ApacheTeaclave](https://twitter.com/ApacheTeaclave).
- See [more](https://teaclave.apache.org/community/).
