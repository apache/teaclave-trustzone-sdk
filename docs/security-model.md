# Security Model

This document describes the trust model, trust boundaries, and security
assumptions of the Apache Teaclave™ TrustZone SDK. It has two audiences:

1. **Developers** writing Trusted Applications (TAs) and Client Applications
   (CAs) with this SDK, who need to know where the security boundary is and what
   responsibilities fall on their code.
2. **Automated reviewers (including LLM-based audit agents)**, who need an
   explicit map of trust postures onto the repository's file structure so that
   findings are calibrated — flagging real boundary issues without raising false
   positives on code where the concern does not apply.

The model here follows the
[GlobalPlatform TEE Internal Core API](https://globalplatform.org/specs-library/tee-internal-core-api-specification/)
and the [OP-TEE](https://www.op-tee.org/) implementation that this SDK is built
on. Nothing in this SDK weakens or replaces those guarantees; it provides
ergonomic Rust bindings on top of them.

> This document describes the security model of the **SDK and its boundary
> code**. Individual demos (e.g. `projects/web3/eth_wallet`) carry their own,
> additional security-assumptions sections scoped to that application. Those are
> not repeated here.

---

## 1. Trust model

A TrustZone system is partitioned into two worlds by hardware:

| | Normal World (REE) | Secure World (TEE) |
|---|---|---|
| Runs | Rich OS (Linux/Android), the CA, all user apps | OP-TEE OS + Trusted Applications |
| Trust posture | **Untrusted** | **Trusted** |
| Memory | Visible to the secure world | Not visible to the normal world |
| In this SDK | `optee-teec` (CA library) | `optee-utee` (TA library) |

### Trusted Computing Base (TCB)

The following are **trusted** and assumed correct:

- The hardware root of trust and TrustZone partitioning.
- The secure boot chain and OP-TEE OS.
- The Trusted Application itself, once loaded and verified by OP-TEE.
- The GlobalPlatform Internal Core API surface OP-TEE provides.

### Adversary

The adversary is assumed to have **full control of the Normal World**,
including root privileges in the Rich OS. Concretely, the adversary can:

- Invoke any TA, open any session, and call any command ID with **arbitrary
  parameters** — values, buffer pointers, and buffer lengths are all
  attacker-chosen.
- Place arbitrary, malicious, or malformed content in any shared-memory buffer
  passed to a TA.
- **Mutate shared-memory buffers concurrently**, including during a TA call
  (Time-of-Check-to-Time-of-Use, TOCTOU).
- Read any memory and any file in the Normal World, including encrypted secure-
  storage blobs that OP-TEE chooses to persist on the Normal World filesystem.
- Delete, withhold, replay, or reorder anything stored or transmitted through
  the Normal World (availability and rollback attacks).
- Observe timing and other side channels visible from the Normal World.

### Out of scope

Unless a specific demo states otherwise, the following are **not** defended
against by this SDK and are the responsibility of the hardware/integrator:

- Physical and hardware attacks (glitching, bus probing, decapsulation).
- Microarchitectural side channels (Spectre-class, cache timing).
- Rollback of secure storage when no anti-rollback hardware (e.g. RPMB) is used.
- Denial of service from the Normal World (it controls scheduling and power).

---

## 2. The trust boundary

The single most important boundary is the **TA entry point**. Everything
crossing from the Normal World into a TA is attacker-controlled until the TA has
validated it.

```
   NORMAL WORLD (untrusted)              ││            SECURE WORLD (trusted)
                                         ││
  ┌──────────────┐   TEEC_InvokeCommand  ││  TA_InvokeCommandEntryPoint
  │  CA / Rich   │ ────────────────────► ││ ──────────────────────────►  TA logic
  │  OS (root)   │   params + shared mem ││  Parameters / ParamMemref     (your code)
  └──────────────┘                       ││
                            TRUST BOUNDARY ↑↑
              (params, pointers, lengths, buffer contents
               are all attacker-controlled and may mutate
               concurrently — validate before use)
```

In this SDK the boundary is crossed through the entry-point macros in
`optee-utee` (`#[ta_invoke_command]`, `#[ta_open_session]`, etc.), which hand
your TA a `Parameters` struct built from the raw `TEE_Param` array.

### Parameter trust, by type

`crates/optee-utee/src/parameter.rs` exposes the GlobalPlatform parameter types:

- **`ValueInput` / `ValueInout`** — two `u32` registers (`a`, `b`) passed by
  value. Untrusted *content*, but bounded in size and not aliased to Normal-World
  memory. Validate the values; there is no pointer/length to worry about.
- **`MemrefInput` / `MemrefInout` / `MemrefOutput`** — a **shared-memory
  reference**: a `{buffer, size}` pair (`raw::Memref`). This is the high-risk
  case. `ParamMemref::buffer()` (`parameter.rs:79`) constructs a Rust slice
  directly over the Normal-World-supplied pointer and size:

  ```rust
  pub fn buffer(&mut self) -> &mut [u8] {
      unsafe { slice::from_raw_parts_mut((*self.raw).buffer as *mut u8, (*self.raw).size) }
  }
  ```

  OP-TEE core guarantees this pointer refers to memory the caller is allowed to
  share (so it cannot be used to read arbitrary secure memory), but **the
  contents and the length are attacker-chosen, and the backing memory remains
  mapped and writable by the Normal World for the duration of the call.**

### Boundary invariants the TA must enforce

These are obligations on **TA code**, not provided automatically:

1. **Validate `param_types` first.** Confirm each slot is the type you expect
   before interpreting it. `as_value`/`as_memref` return `BadParameters` on a
   type mismatch — propagate that, do not bypass it.
2. **Treat every byte of a memref as adversarial input.** Length, encoding, and
   structure must all be checked. Never assume a buffer is NUL-terminated, well-
   formed, or non-empty.
3. **Bound all lengths.** A `size` of `0`, a huge `size`, or a `size` that does
   not match the payload are all legal inputs the adversary may send.
4. **Copy-then-validate to avoid TOCTOU.** Because the Normal World can mutate a
   shared buffer concurrently, copy untrusted input into secure memory **once**
   before validating and using it. Do not read the same shared field twice and
   assume it is unchanged (a "double fetch"). Treat `MemrefInput` as read-once.
5. **Do not leak secrets through `*Output` / `*Inout` buffers.** Anything written
   to an output memref becomes visible to the Normal World. Write only what the
   caller is authorized to learn; size outputs deliberately and set the updated
   size with `set_updated_size`.
6. **Fail closed.** On any validation failure, return an error
   (`ErrorKind::BadParameters` and friends) — never proceed with partially
   validated input.

---

## 3. Trust-posture map of the repository

Use this table to decide whether a given concern (especially "untrusted input")
applies to a file. This is the key reference for an automated reviewer.

| Path | World / role | Trust posture | What to scrutinize |
|---|---|---|---|
| `crates/optee-utee/` | TA library (Secure World) | **Boundary + trusted** | The boundary lives here. `parameter.rs`, `tee_parameter.rs`, the entry-point glue, and any `unsafe` deref of caller-supplied pointers/lengths. |
| `crates/optee-utee-macros/` | TA entry-point codegen | **Boundary** | The generated `TA_*EntryPoint` wrappers: do they faithfully pass `param_types` and the raw params, and propagate errors without dropping them? |
| `crates/optee-utee-sys/` | Raw FFI to OP-TEE Internal Core API | **Below the type system** | Signatures must match OP-TEE; `unsafe` correctness. The Rust type system does **not** protect callers here. |
| `crates/optee-teec/` | CA library (Normal World) | **Untrusted side** | This runs in the adversary's world. "Missing input validation" here is generally **not** a TA-security finding — the TA cannot trust this code regardless. Focus instead on memory safety and not mishandling secrets returned to the CA. |
| `crates/optee-teec-sys/` | Raw FFI for the CA | Untrusted side / FFI | FFI correctness only. |
| `crates/secure_db/`, `crates/rustls_provider/` | Run inside the TA | **Trusted, but process untrusted data** | Logic runs in the TEE, but inputs (DB contents persisted via Normal World storage, bytes from a TLS peer) originate outside the TCB. Apply the boundary invariants to those inputs. |
| `crates/*-build`, `*-macros`, `*-systest` | Build-time / test tooling | Build-time | Not in the runtime TCB. Review as ordinary tooling, not as boundary code. |
| `examples/` | Illustrative TA+CA pairs | **Illustrative, not hardened** | Demonstrate API usage. They are teaching material and may intentionally omit production hardening; do not report them as if they were production code, but *do* note where they model an unsafe pattern a developer might copy. |
| `projects/` | Reference applications (e.g. `web3/eth_wallet`) | **Reference, with stated assumptions** | Read the project's own "Security Assumptions" section first; review against *that* stated threat model. |
| `tests/`, `.patches/`, `tools/` | Test harness / tooling | Build/test-time | Not runtime TCB. |

Within any TA-side crate, the layout of a typical application is:

- **`ta/`** — runs in the Secure World. The trust boundary is its entry points.
- **`host/` (CA)** — runs in the Normal World. Untrusted.
- **`proto/`** — shared message/serialization definitions used by both sides.
  Deserialization of these structures **inside the TA** is boundary code: a
  malicious CA can send malformed `proto` bytes.

---

## 4. Storage, secrets, and other assumptions

- **Secure storage is confidential and integrity-protected, but not inherently
  anti-rollback or highly available.** OP-TEE may persist secure objects as
  encrypted blobs on the Normal World filesystem, where the adversary can delete
  or roll them back. Anti-rollback requires hardware support such as RPMB.
  (See the `eth_wallet` demo's notes for a concrete discussion.)
- **Secrets must never cross to the Normal World in cleartext** unless the
  application's threat model explicitly accepts it. Returning a mnemonic or key
  to the CA is a deliberate, documented risk where it appears in demos.
- **A secure user interface (trusted display/input) is hardware-specific and not
  provided by this SDK.** Where a flow needs user confirmation of a sensitive
  action, that confirmation cannot be trusted if it round-trips through the
  Normal World.
- **Cryptographic operations** should use the OP-TEE/GlobalPlatform crypto API
  surface (`crates/optee-utee/src/crypto_op.rs`, `arithmetical.rs`) rather than
  re-implementing primitives in the TA.

---

## 5. Dependencies and the supply chain

This is a TrustZone-specific concern that differs sharply from ordinary
applications: **every crate compiled into a TA runs inside the Secure World and
is therefore part of the Trusted Computing Base.** A vulnerability or backdoor
in a TA dependency is not "just" code execution in a userspace process — it is
code execution **inside the TEE**, with access to whatever secrets and
capabilities the TA holds. The trust boundary of §2 stops attacker *input* at
the entry point; it does **not** sandbox the TA's own dependencies.

### What runs where

| Dependency kind | Executes | Trust domain |
|---|---|---|
| Regular `[dependencies]` of a TA | At runtime, **inside the TEE** | **TCB** — fully trusted, no sandbox |
| `[dependencies]` of a CA | At runtime, in the Normal World | Untrusted world (not TA-security-relevant) |
| `[build-dependencies]` and proc-macros (`*-macros`) | At **build time** on the developer/CI host | Build host — build-time code execution |

### Consequences for the audit

- **The TCB includes the full transitive dependency tree of each TA.** When
  reviewing a TA, the in-scope code is not only `ta/` — it is every crate the TA
  pulls in. Cryptographic crates such as `rustls`, `rustls-rustcrypto`,
  `ed25519-dalek`, `secp256k1`, `sha3`, `bip32`, and serialization crates such
  as `serde`/`bincode` (see `crates/rustls_provider`, `crates/secure_db`, and
  `projects/web3/eth_wallet/ta`) all run with full TEE privilege.
- **`no-std` vs `std` is a TCB-size decision, not only an ergonomics one.** The
  SDK recommends `no-std` by default; one security reason is that `std` mode
  pulls in a substantially larger dependency and runtime surface, all of which
  enters the TCB. Prefer the smallest dependency set that meets the requirement.
- **Software crypto in a TA bypasses hardware crypto and key isolation.** Crates
  like `ed25519-dalek`/`secp256k1` perform key operations in TA-resident memory
  rather than in a hardware crypto engine or secure key store. That is a
  legitimate design choice, but it means key material lives in TEE RAM and the
  implementation's side-channel resistance is now your concern. Where the
  platform offers it, prefer the OP-TEE/GlobalPlatform crypto API
  (`crates/optee-utee/src/crypto_op.rs`).
- **Randomness must be TEE-backed.** A TA's RNG (e.g. anything reaching
  `getrandom`) must be sourced from a TEE/hardware entropy source, never from
  the Normal World. Verify the entropy path for any crate that needs randomness
  inside a TA.
- **Build-time dependencies are a supply-chain risk even though they never enter
  the TEE.** Proc-macros and build scripts execute arbitrary code on the build
  host and can tamper with the generated TA binary. Treat the build host and CI
  as part of the trust chain.

### Current state and recommendations

The repository does not currently ship a dependency-vetting policy (no
`deny.toml`, `cargo-audit`/`cargo-vet`/`cargo-supply-chain` configuration, and no
committed top-level lockfile policy for downstream TAs). For
production-grade TAs, integrators should:

- **Pin and lock** dependencies (commit `Cargo.lock`) so audited versions are
  reproducible.
- Run **`cargo audit`** (RUSTSEC advisories) and consider **`cargo deny`** to
  enforce allowed licenses, ban yanked/duplicate crates, and gate new
  dependencies.
- Consider **`cargo vet`** or vendoring for the subset of crates that end up in
  the TCB.
- **Minimize the TA dependency tree** and review new TA dependencies as TCB
  additions, with the same scrutiny as first-party TA code.

---

## 6. Guidance for automated / LLM security review

> **Note to the reviewing agent:** When you audit a TA, **do not stop at the
> first-party `ta/` code — also review its dependencies whenever your tooling and
> context allow it.** A TA's entire transitive crate tree runs inside the TEE and
> is part of the TCB (§5). If you can access the dependency sources or lockfile,
> enumerate the TA's transitive dependencies, flag any that are unpinned,
> yanked, or carry known RUSTSEC advisories, and pay particular attention to
> crates that handle crypto, randomness/entropy, or deserialization inside the
> TEE. If you cannot reach the dependency sources, say so explicitly and list the
> TA's declared dependencies as unreviewed TCB surface rather than silently
> skipping them.

To keep findings high-signal:

**Where real findings concentrate**
- TA entry points and anything reachable from them that reads `Parameters`.
- Every `ParamMemref::buffer()` use: is the length bounded? Is the content
  validated? Is it read exactly once (no double-fetch / TOCTOU)?
- `unsafe` blocks in `optee-utee` and the `*-sys` crates that dereference
  caller-supplied pointers or lengths.
- Deserialization inside a TA of `proto`/shared structures.
- Output/Inout memrefs that might leak more than intended.
- Secure-storage code that assumes persistence, freshness, or availability.
- TA dependencies (§5): the in-scope TCB is the TA's full transitive crate tree,
  not just `ta/` — including software crypto, the entropy/RNG path, and `serde`/
  `bincode` deserialization of attacker-influenced data inside the TEE.

**Expected non-findings (avoid these false positives)**
- "Missing input validation in the CA" — CA code (`optee-teec`, `host/`) is in
  the untrusted world; the TA must validate regardless, so CA-side validation is
  not a security control.
- Treating `examples/` as production code. Note copy-risk patterns, but frame
  them as illustrative.
- Flagging `unsafe` in `*-sys` crates merely for existing — FFI is `unsafe` by
  necessity. The finding must be a concrete mismatch or misuse.
- Reporting a demo's *documented and accepted* risk (e.g. mnemonic returned to
  the Normal World in `eth_wallet`) as a new vulnerability.

**Before reporting**, state which side of the trust boundary the code runs on
and which adversary capability (§1) the issue depends on. If a finding does not
trace to a concrete adversary capability crossing the boundary, it is likely a
false positive.

---

## 7. Reporting vulnerabilities

Security issues in the SDK itself should be reported privately first, per
[`SECURITY.md`](../SECURITY.md), before any public disclosure.
