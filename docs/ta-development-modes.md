---
permalink: /trustzone-sdk-docs/ta-development-modes.md
---

# TA Development Modes

## Comparison

### `no-std`

- **Pros**:
  - Reuses standard Rust tier-1 toolchain targets (`aarch64-unknown-linux-gnu`, 
    `arm-unknown-linux-gnueabihf`).
  - Significant performance improvements.
  - Substantial reduction in binary size.
  
- **Cons**:
  - Limited support for third-party crates. In the no-std mode, Trusted
    Applications (TAs) are unable to utilize crates dependent on the standard
    library (std).

### `std`

- **Pros**:
  - Enables the utilization of more third-party crates, including those
    requiring `std`, such as `rustls`, which are essential for functionality.
  
- **Cons**:
  - Manual porting of `std` with infrequent updates. Currently using `std`
    version `1.80.0` and `Rust` version `nightly-2024-05-14`, which might not
    meet the MSRV requirements of some crates.

## Supported Examples

- **Common**: See
  [Overview of OP-TEE Rust Examples](https://teaclave.apache.org/trustzone-sdk-docs/overview-of-optee-rust-examples/).

- **`no-std`**: Excludes `test_tls_client`, `test_tls_server`, `test_secure_db_abstraction`.

- **`std`**: Excludes `test_mnist_rs`, `test_build_with_optee_utee_sys`. 