# crates.io Ownership Record

This file records the crates the Apache Teaclave TrustZone SDK publishes on
crates.io, what each crate does, and who owns it there. Use it as the reference
when checking or changing crate ownership.

## Crates

| Crate | Description |
| ----- | ----------- |
| [`optee-teec`] | TEE Client API for CAs in the normal world |
| [`optee-teec-build`] | Build-script helper for OP-TEE supplicant plugins |
| [`optee-teec-macros`] | Procedural macros for supplicant plugin entry points |
| [`optee-teec-sys`] | Raw FFI bindings to `libteec` and the plugin ABI |
| [`optee-utee`] | TEE Internal Core API for TAs in the secure world |
| [`optee-utee-build`] | Build-script helper that generates TA headers and linker configuration |
| [`optee-utee-macros`] | Procedural macros for TA entry points |
| [`optee-utee-sys`] | Raw FFI bindings to `libutee`, TA syscalls, and TEE sockets |
| [`optee-utee-mock`] | **Deprecated**; replaced by the `mock` feature used by `optee-utee` |

> **`optee-utee-mock` is no longer used.** Its mocking functionality has been
> folded into the `mock` feature of [`optee-utee-sys`], which [`optee-utee`]
> enables for host-side unit tests.

`optee-teec-systest`, `rustls_provider`, and `secure_db` are not published
(`publish = false`); the `cargo-optee` tool is not published either.

## Maintainers & crates.io owners

| Crate | Owners (crates.io / GitHub handle) |
| ----- | ---------------------------------- |
| [`optee-teec`] | [mssun], [DemesneGH] |
| [`optee-teec-build`] | [DemesneGH] |
| [`optee-teec-macros`] | [mssun], [DemesneGH] |
| [`optee-teec-sys`] | [mssun], [DemesneGH] |
| [`optee-utee`] | [mssun], [DemesneGH] |
| [`optee-utee-build`] | [DemesneGH] |
| [`optee-utee-macros`] | [mssun], [DemesneGH] |
| [`optee-utee-sys`] | [mssun], [DemesneGH] |
| [`optee-utee-mock`] | [DemesneGH] |

All crates are owned by individual accounts; there is no shared or team owner.

## Publishing

Releases currently use a crates.io API token (`cargo login`), as described in
[Release Tips](release-tips.md#publish-on-cratesio). In the future, publishing
may move to OIDC-based trusted publishing from CI, which avoids storing a
long-lived token; crate ownership will still control who can configure and
manage the crates.

[mssun]: https://github.com/mssun
[DemesneGH]: https://github.com/DemesneGH

[`optee-teec`]: https://crates.io/crates/optee-teec
[`optee-teec-build`]: https://crates.io/crates/optee-teec-build
[`optee-teec-macros`]: https://crates.io/crates/optee-teec-macros
[`optee-teec-sys`]: https://crates.io/crates/optee-teec-sys
[`optee-utee`]: https://crates.io/crates/optee-utee
[`optee-utee-build`]: https://crates.io/crates/optee-utee-build
[`optee-utee-macros`]: https://crates.io/crates/optee-utee-macros
[`optee-utee-sys`]: https://crates.io/crates/optee-utee-sys
[`optee-utee-mock`]: https://crates.io/crates/optee-utee-mock
