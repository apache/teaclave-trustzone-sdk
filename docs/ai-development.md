# AI Development Guide

This document describes repository-specific conventions and workflows for AI coding assistants working on the Apache Teaclave TrustZone SDK.

This guide is a snapshot of repository conventions and workflows. When it conflicts with the current repository configuration, scripts, or CI workflows, the repository's current state takes precedence.

## 1. Repository Layout and Toolchain

### 1.1 Repository Layout

- `crates/`: Rust crates for the OP-TEE client and trusted execution environments (TEEs), including tests and system tests.
- `tools/`: host-side development tools, including `cargo-optee`.
- `examples/`: examples built through the repository's example Makefile targets.
- `projects/`: standalone projects with their own build workflows. For example, `projects/web3/eth_wallet` is built separately from `make std-examples`.
- `tests/`: QEMU setup, launch, and test scripts.
- `ci/`: CI helper scripts.
- `.github/workflows/`: CI workflow definitions.
- `scripts/`: repository helper scripts.
- `docs/`: project documentation, including the security model and this guide.
- Root-level `*.json` files: target configuration files, including OP-TEE target specifications.
- `SECURITY.md`: security vulnerability reporting instructions.

**Example integration:** when changing a crate's public API or auto-trait behavior such as `Send` or `Sync`, build the affected examples locally as well as testing the crate. Workspace tests alone may not catch example-level integration failures.

### 1.2 Rust Toolchain

The repository's `rust-toolchain.toml` specifies:

- Channel: `nightly-2026-08-05`
- Profile: `minimal`
- Components: `rust-src`, `rustfmt`, `clippy`
- Targets: `aarch64-unknown-linux-gnu` and `arm-unknown-linux-gnueabihf`

Use the repository-pinned toolchain unless a task explicitly requires investigating another version. Do not casually update the toolchain or target configuration as part of an unrelated change.

## 2. General Development Principles

- Inspect the existing implementation, callers, trait implementations, feature configuration, and tests before making changes.
- Prefer the smallest coherent change that fixes the problem while preserving existing API and runtime behavior.
- When changing an API or shared abstraction, update the related call sites and implementations in the same change. Check relevant feature combinations and downstream examples.
- Follow existing naming, error-handling, module organization, and test conventions. Avoid unrelated refactoring or new dependencies without a clear need.

## 3. Dependencies, Features, and Build Configuration

- Check the relevant `Cargo.toml` files before changing dependencies or features. Cargo feature unification and optional dependencies can affect multiple crates in a workspace.
- Do not assume a dependency is always enabled or a feature is enabled only for the crate being edited. Inspect feature definitions and the build paths that activate them.
- If a feature-specific implementation replaces or excludes a dependency, consider both enabled and disabled configurations relevant to the change.
- Keep host-side and TEE-side dependencies distinct. A crate that builds on the host may require a different target, toolchain, or environment when built for the TEE.
- `STD=y` affects environment setup. Set it before sourcing `environment` when configuring standard-library examples; see §8.3.
- Avoid changing lockfiles or broad dependency constraints unless required by the task.
- Keep `*-sys` crates as thin mappings of the underlying OP-TEE C interfaces; do not add unrelated high-level abstractions to them.

## 4. Unsafe Rust, Concurrency, and Resource Management

### 4.1 Unsafe Rust

- Keep `unsafe` blocks as small as practical and document the safety invariant they rely on.
- A safety comment should explain why the operation is valid, including relevant pointer validity, lifetime, aliasing, initialization, ownership, or synchronization requirements.
- Before changing an unsafe abstraction, identify which invariants the abstraction guarantees and which remain the caller's responsibility.
- Do not add manual `Send` or `Sync` implementations merely to silence a compiler error. Determine whether the underlying state can safely be accessed across threads and whether the ownership model supports the claimed auto traits.

### 4.2 Concurrency and Interior Mutability

- Trace all access paths to shared state before changing synchronization or ownership.
- `Arc` provides shared ownership; it does not by itself make the contained type thread-safe.
- `Mutex::get_mut` does not acquire the mutex at runtime. Its use relies on exclusive access to the mutex itself and is not equivalent to locking it.
- When changing `Send` or `Sync` behavior, consider every field and the invariants of the whole type, not only the immediate compiler error.
- Preserve lock ordering and avoid deadlocks, data races, or unexpected blocking. Check whether callbacks, cleanup paths, and error handling can re-enter code that uses the same lock.

### 4.3 FFI and Resource Lifecycles

- Check the C API contract and the Rust wrapper's ownership model together when modifying FFI code.
- Make resource ownership and cleanup responsibilities explicit. Avoid double-closing, use-after-close, dangling pointers, and mismatches between Rust lifetimes and native resource lifetimes.
- For OP-TEE resources such as contexts, sessions, and shared memory, consider initialization, use, error paths, and teardown.
- When a wrapper stores a handle indirectly through shared state, ensure every operation uses the correct state and cleanup cannot invalidate resources still in use.
- Do not assume a raw handle remains valid merely because its Rust wrapper still exists. Check the wrapper's lifecycle and synchronization guarantees.

### 4.4 Error Handling

- Preserve meaningful error information across FFI and abstraction boundaries.
- Do not silently discard errors from initialization, resource acquisition, invocation, or cleanup when they affect correctness.
- Follow the surrounding API's error-handling conventions. Avoid introducing panics into paths expected to return errors, especially in low-level or security-sensitive code.

## 5. Security-Sensitive Changes

For security-sensitive changes, read [`SECURITY.md`](../SECURITY.md) and the repository's [`security-model.md`](security-model.md).

Pay particular attention to:

- Trust boundaries between the normal world and the trusted execution environment.
- Input validation, authorization, and assumptions about caller-controlled data.
- Memory safety, resource ownership, concurrency, and FFI contracts.
- Cryptographic API usage, key material handling, and relevant error paths.
- Whether a change weakens an existing security invariant or changes the documented threat model.

Keep the implementation, tests, and relevant security documentation consistent.

## 6. Tests and Verification Evidence

### 6.1 Reporting Verification Results

Use these status labels consistently:

- **Passed**: the specified command completed successfully.
- **Failed**: the command ran and reported a failure.
- **Not run**: the command was not executed.
- **Blocked**: a prerequisite or environmental limitation prevented the command from running.

Report the exact command and its result, and identify the scope of the check. A crate test, target build, example build, QEMU runtime test, and full CI workflow are distinct checks.

Do not report a check as passed based only on reading its configuration or reasoning about the code. Distinguish compile/build checks from runtime tests.

### 6.2 Unit-Test Conventions

- Every unit test must include a `Guarantee:` or `Guarantees:` comment describing the behavior or invariant it is intended to establish.
- The guarantee must match the assertions and test setup. Update it when the test's behavior changes.
- Prefer tests that verify observable behavior, boundary conditions, failure paths, and resource lifecycle semantics.
- Add focused regression tests for changed behavior, particularly when modifying shared code or error paths.

## 7. Formatting and Linting

Run formatting and linting from the relevant workspace directories.

### 7.1 Formatting

For `crates/`:

```bash
cd crates
cargo fmt --all --check
```

For `tools/`:

```bash
cd tools
cargo fmt --all --check
```

### 7.2 Clippy

For `crates/`:

```bash
cd crates
cargo clippy \
  --workspace \
  --exclude secure_db \
  --exclude rustls_provider \
  --target aarch64-unknown-linux-gnu \
  -- -D warnings
```

For the `cargo-optee` tool:

```bash
cd tools
cargo clippy -p cargo-optee -- -D warnings
```

The exclusions and target are part of the repository's established command.

### 7.3 License Headers

The repository's CI checks license headers using `apache/skywalking-eyes/header@v0.8.0` and `.licenserc.yaml`. Follow the configured header format when adding source files.

## 8. Build and Test Workflows

CI workflow definitions are under `.github/workflows/`. Consult the relevant workflow and its invoked scripts when reproducing a CI job locally.

These workflows depend on the local OP-TEE toolchain and environment. Consult the repository scripts and relevant CI job for additional prerequisites.

### 8.1 SDK Build and Tests

A common setup path from the repository root is:

```bash
./setup.sh
source "$HOME/.cargo/env"
./build_optee_libraries.sh "$HOME"
export OPTEE_DIR="$HOME"
source environment
```

This is not the only possible environment configuration. Check `environment` and the setup scripts for the expected values of `TA_DEV_KIT_DIR`, `OPTEE_CLIENT_EXPORT`, `OPTEE_DIR`, `TARGET_TA`, and `TARGET_HOST`. A custom installation may require explicit environment values.

The `environment` script derives `TARGET_TA` from `ARCH_TA` and `STD`: `ARCH_TA` selects the target architecture (`arm` or `aarch64`), while `STD` selects the target ABI suffix (`-optee` or `-linux-gnu`). To force re-derivation, unset `TARGET_TA` and `TARGET_HOST` before sourcing `environment`. Check the script for the exact mapping and any additional prerequisites.

Relevant cross-compilation commands used by CI include:

```bash
cd crates
cargo build -p optee-utee --target aarch64-unknown-linux-gnu
cargo build -p optee-teec --target aarch64-unknown-linux-gnu
```

Relevant crate tests include:

```bash
cd crates
cargo test -p optee-utee --features no_panic_handler
cargo test -p optee-teec
cargo test -p optee-utee-build
cargo test -p optee-utee --features std
```

To run the CI helper script, execute it from the `ci/` directory because it uses relative paths:

```bash
(cd ci && ./ci.sh)
```

### 8.2 System Tests

The `optee-teec-systest` system test is run as a CI job on an ARM64 host using the ARM64 OP-TEE client libraries:

```bash
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$OPTEE_DIR/optee_client/export_arm64/usr/lib"
cd crates
cargo run -p optee-teec-systest
```

The UTEE system-test build can be checked with:

```bash
cd crates
cargo build -p optee-utee-systest
```

The TEEC system test runs on the host, while the UTEE command builds the trusted-side test crate.

### 8.3 Examples and QEMU

Build no-std examples from the repository root:

```bash
make no-std-examples -j2
```

For standard-library examples, configure dependencies and environment in this order:

```bash
./setup_std_dependencies.sh
export STD=y
source environment
make std-examples -j2
```

`environment` derives `TARGET_TA` based on `STD`. If it was previously sourced while `STD` was unset or configured for no-std, source it again after setting `STD=y`.

The standalone Ethereum wallet project has its own build path:

```bash
cd projects/web3/eth_wallet
make
```

It is not included in `make std-examples`.

For QEMU runtime testing, inspect the relevant entry points and prerequisites:

```bash
cat ci/ci.sh
cat tests/setup.sh
cat tests/optee-qemuv8.sh
```

The repository's QEMU test scripts include `tests/test_*.sh`. Follow their current usage to select the setup, launcher, and relevant test invocation.


If reproducing the `test-build-on-dev-docker` CI job, inspect its steps in `ci.yml`, including configuration switching and the `$RUST_STD_DIR` symlink setup.

### 8.4 Tools

Run the `cargo-optee` tests from `tools/`:

```bash
cd tools
cargo test -p cargo-optee
```

When changing a tool, inspect its affected CLI paths and any build scripts or projects that consume its output.

## 9. Code Review and Collaboration

- Review the full diff, not only the lines changed in the last edit.
- Check for unintended changes to generated files, lockfiles, formatting, feature configuration, and unrelated code.
- Ensure API changes are reflected in callers, trait implementations, tests, examples, and documentation where applicable.
- Keep changes focused so they can be reviewed and reverted independently.

## 10. Git and Delivery

### 10.1 Commit Messages

Follow the Linux kernel style, adapted to this repository.

- **Subject:** `subsystem: imperative summary`, for example
  `optee-teec: fix session cleanup` or `docs: clarify build instructions`.
  Use the affected crate, area, or `docs`/`ci`/`examples` as the prefix.
  Target 50 characters, do not exceed 72, and do not end it with a period.
- **Blank line:** separate the subject from the body with exactly one blank
  line.
- **Body:** wrap at 72 characters. Explain why the change is needed and any
  constraints or prior behavior that are not obvious from the diff. The diff
  already shows what changed, so do not repeat it line by line.
- **Mood:** use the imperative in the subject and body (`add`, `fix`,
  `remove`; not `added`/`fixes`).
- **Length:** a subject alone is fine for trivial changes; add a body when
  the motivation, tradeoffs, or references need recording.

Example:

```text
optee-teec: fix session cleanup

Close the session before dropping the context. Otherwise a failed
invocation leaves the TA session open until process exit.
```

### 10.2 Before Committing

- Inspect `git status` and review the staged diff.
- Stage only files that belong to the intended change.
- Do not include unrelated local modifications or generated artifacts.
- If the task requires a commit, ensure it contains the intended changes and has an appropriate subject.

## 11. Final Report

Summarize the work concisely:

- **Changes:** main code, test, or documentation changes.
- **Verification:** commands run and statuses, using §6 definitions.
- **Limitations:** important checks that remain unperformed or environmental constraints affecting the result.
