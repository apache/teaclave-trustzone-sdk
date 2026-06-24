---
permalink: /trustzone-sdk-docs/optee-utee-parameter-migration.md
---

# Migrating TA Code to the Typed Parameter API

This guide explains how to migrate OP-TEE TA code from the deprecated
`optee_utee::Parameters` API to the typed parameter API. The public types are
re-exported from `optee_utee` and `optee_utee::prelude`.

The typed API keeps the same OP-TEE parameter model: every entry point receives
up to four parameters, and each slot has one runtime type tag. The difference is
that TA code now represents those tags with Rust types such as
`ParameterMemrefInput<'_>`, `ParameterValueOutput<'_>`, and `ParameterNone`.
The Normal World client application may be implemented with any OP-TEE client
stack; this document only describes the Secure World TA side.

## Why migrate

The old API exposed each slot as a generic `Parameter` and required unsafe
accessors:

```rust
let mut p0 = unsafe { params.0.as_memref()? };
let buf = p0.buffer();
```

That code allowed TA-side code to request a value or memref dynamically at each
use site. A mismatch was discovered only when that unsafe accessor ran, or
worse, hidden behind mutable access to a buffer whose intended direction was
not visible in the Rust type.

The new API makes the expected direction explicit:

```rust
let input = p0.as_memref_input()?.get_buffer();
let output = p1.as_memref_output()?;
output.set_output(bytes)?;
```

This does not make the client application and TA parameter layouts a
compile-time contract. They are built as separate OP-TEE components, and the TEE
still delivers parameter type tags at runtime. The typed API improves the TA
side: it records the TA's expected layout in Rust types, performs consistent
runtime validation at the entry point or branch accessor, and removes most
call-site unsafe code.

For commands with a fixed parameter layout, the entry-point signature can use a
concrete 4-tuple. For commands where different command IDs use different
layouts, use `ParametersAny<'_>` and validate each slot inside the command
branch.

## Imports

Most TA code should import the prelude:

```rust
use optee_utee::prelude::*;
use optee_utee::{ErrorKind, Result};
```

The prelude exports the TA entry-point macros, `ParametersAny`,
`ParametersNone`, all typed parameter wrappers, and the read/write traits needed
for methods such as `get_buffer`, `get_a`, and `set_output`.

## Parameter Directions

Choose typed wrappers from the TA's perspective:

| Data flow | TA typed parameter |
| --- | --- |
| No parameter in this slot | `ParameterNone` |
| Client application provides two `u32` values for the TA to read | `ParameterValueInput` |
| TA writes two `u32` values back to the client application | `ParameterValueOutput<'_>` |
| TA reads and writes two `u32` values | `ParameterValueInout<'_>` |
| Client application provides a buffer for the TA to read | `ParameterMemrefInput<'_>` |
| TA writes bytes into a client-provided output buffer | `ParameterMemrefOutput<'_>` |
| TA reads and writes the same buffer | `ParameterMemrefInout<'_>` |

The direction names are strict. If the TA writes to a buffer, use
`ParameterMemrefOutput<'_>` or `ParameterMemrefInout<'_>`. If the TA only reads
from a buffer, use `ParameterMemrefInput<'_>` or `ParameterMemrefInout<'_>`.

## Entry-Point Signatures

The `#[ta_open_session]` and `#[ta_invoke_command]` macros accept any parameter
type that implements `FromRawParameters<'_>`. In normal code this means either:

1. `ParametersNone`
2. `ParametersAny<'_>`
3. A concrete 4-tuple of typed wrappers

### No Parameters

Use `ParametersNone` when no parameters are expected.

```rust
#[ta_open_session]
fn open_session(_params: &mut ParametersNone) -> Result<()> {
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(cmd_id: u32, _params: &mut ParametersNone) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Ping => Ok(()),
        _ => Err(ErrorKind::BadParameters.into()),
    }
}
```

### Fixed Layout

Use a concrete tuple when every supported command for an entry point uses the
same parameter layout. The macro converts the raw OP-TEE parameters into the
tuple before your function runs, and returns `TEE_ERROR_BAD_PARAMETERS` if any
runtime slot type does not match the tuple.

```rust
#[ta_invoke_command]
fn invoke_command(
    cmd_id: u32,
    (input, output, _, _): &mut (
        ParameterMemrefInput<'_>,
        ParameterMemrefOutput<'_>,
        ParameterNone,
        ParameterNone,
    ),
) -> Result<()> {
    let bytes = input.get_buffer();
    let response = handle_request(Command::from(cmd_id), bytes)?;
    output.set_output(response)
}
```

Use this style for examples such as a single request buffer plus a single
response buffer.

### Command-Dependent Layout

Use `ParametersAny<'_>` when different command IDs use different layouts. Each
slot is first decoded into `ParameterAny`, and command branches then request the
specific type they expect.

```rust
#[ta_invoke_command]
fn invoke_command(cmd_id: u32, (p0, p1, p2, _): &mut ParametersAny<'_>) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Update => {
            let input = p0.as_memref_input()?.get_buffer();
            digest_update(input);
            Ok(())
        }
        Command::DoFinal => {
            let input = p0.as_memref_input()?.get_buffer();
            let output = p1.as_memref_output()?;
            let size = digest_final(input, output.get_buffer_mut())?;
            p2.as_value_output()?.set_a(size as u32);
            output.set_updated_size(size)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}
```

This is the right choice for cryptographic examples where `Prepare`, `Update`,
and `Final` commands use different parameter directions.

## Replacing Value Access

Old code:

```rust
let value = unsafe { params.0.as_value()? };
let mode = value.a();
let flags = value.b();
```

New code for an input value:

```rust
let value = p0.as_value_input()?;
let mode = value.get_a();
let flags = value.get_b();
```

New code for an output value:

```rust
let value = p0.as_value_output()?;
value.set_a(result_len as u32);
value.set_b(status);
```

New code for an in/out value:

```rust
let value = &mut params.0;
value.set_a(value.get_a() + 1);
```

The trait split is intentional:

| Operation | Trait | Implemented by |
| --- | --- | --- |
| `get_a`, `get_b` | `ParameterValueRead` | `ParameterValueInput`, `ParameterValueInout` |
| `set_a`, `set_b` | `ParameterValueWrite` | `ParameterValueOutput`, `ParameterValueInout` |

## Replacing Memref Access

Old code:

```rust
let mut p0 = unsafe { params.0.as_memref()? };
let input = p0.buffer();
```

New code for an input memref:

```rust
let input = p0.as_memref_input()?.get_buffer();
```

Old code for output:

```rust
let mut p1 = unsafe { params.1.as_memref()? };
let output = p1.buffer();
output[..bytes.len()].copy_from_slice(&bytes);
p1.set_updated_size(bytes.len());
```

New code:

```rust
let output = p1.as_memref_output()?;
output.set_output(bytes)?;
```

Use `set_output` for the common case where the output starts at offset 0. Use
`write_at` when appending or writing at a specific offset:

```rust
let output = p0.as_memref_output()?;
output.write_at(0, header)?;
output.write_at(header.len(), body)?;
```

Use `get_buffer_mut` when an API writes directly into the output buffer:

```rust
let output = p1.as_memref_output()?;
let written = cipher.update(input.get_buffer(), output.get_buffer_mut())?;
output.set_updated_size(written)
```

When using `get_buffer_mut`, always call `set_updated_size` afterward.
Otherwise the client application may observe an incorrect output size.

The memref traits are:

| Operation | Trait | Implemented by |
| --- | --- | --- |
| `get_buffer` | `ParameterMemrefRead` | `ParameterMemrefInput`, `ParameterMemrefInout` |
| `get_buffer_mut`, `set_updated_size`, `set_output`, `write_at` | `ParameterMemrefWrite` | `ParameterMemrefOutput`, `ParameterMemrefInout` |

## Open Session Parameters

Session opening is migrated the same way as command invocation.

If the TA expects no parameters during session opening, use `ParametersNone`:

```rust
#[ta_open_session]
fn open_session(_params: &mut ParametersNone) -> Result<()> {
    Ok(())
}
```

If the TA expects open-session parameters, express that expected layout with a
concrete tuple or `ParametersAny<'_>`. For example, this TA expects slot 0 to be
a client-provided input buffer containing an `f64` learning rate:

```rust
#[ta_open_session]
fn open_session(
    (p0, _, _, _): &mut (
        ParameterMemrefInput<'_>,
        ParameterNone,
        ParameterNone,
        ParameterNone,
    ),
) -> Result<()> {
    let learning_rate = f64::from_le_bytes(
        p0.get_buffer()
            .try_into()
            .map_err(|_| ErrorKind::BadParameters)?,
    );
    init(learning_rate)
}
```

When the runtime parameters do not match this tuple, the generated entry point
returns `TEE_ERROR_BAD_PARAMETERS` before calling `open_session`.

## Session Context

The parameter migration does not change session-context handling. If the old
entry point had a context parameter, keep it and only change the parameter type:

```rust
#[ta_open_session]
fn open_session(_params: &mut ParametersNone, ctx: &mut AesCipher) -> Result<()> {
    *ctx = AesCipher::default();
    Ok(())
}

#[ta_invoke_command]
fn invoke_command(
    ctx: &mut AesCipher,
    cmd_id: u32,
    params: &mut ParametersAny<'_>,
) -> Result<()> {
    dispatch(ctx, cmd_id, params)
}
```

The context type must still implement `Default` for `#[ta_open_session]` with a
context parameter.

## Complete Migration Example

Old code:

```rust
#[ta_invoke_command]
fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Serialize => {
            let mut p0 = unsafe { params.0.as_memref()? };
            let output = p0.buffer();
            let bytes = serde_json::to_vec(&Point { x: 1, y: 2 })
                .map_err(|_| ErrorKind::BadParameters)?;

            if bytes.len() > output.len() {
                p0.set_updated_size(bytes.len());
                return Err(ErrorKind::ShortBuffer.into());
            }

            output[..bytes.len()].copy_from_slice(&bytes);
            p0.set_updated_size(bytes.len());
            Ok(())
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}
```

New code:

```rust
#[ta_invoke_command]
fn invoke_command(cmd_id: u32, (p0, _, _, _): &mut ParametersAny<'_>) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Serialize => {
            let output = p0.as_memref_output()?;
            let bytes = serde_json::to_vec(&Point { x: 1, y: 2 })
                .map_err(|_| ErrorKind::BadParameters)?;
            output.set_output(bytes)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}
```

If `Serialize` is the only command and the layout is fixed, the TA entry-point
validation can be made stricter:

```rust
#[ta_invoke_command]
fn invoke_command(
    cmd_id: u32,
    (p0, _, _, _): &mut (
        ParameterMemrefOutput<'_>,
        ParameterNone,
        ParameterNone,
        ParameterNone,
    ),
) -> Result<()> {
    match Command::from(cmd_id) {
        Command::Serialize => {
            let bytes = serde_json::to_vec(&Point { x: 1, y: 2 })
                .map_err(|_| ErrorKind::BadParameters)?;
            p0.set_output(bytes)
        }
        _ => Err(ErrorKind::BadParameters.into()),
    }
}
```

## Migration Checklist

1. Replace `use optee_utee::{..., Parameters, ...}` with
   `use optee_utee::prelude::*`.
2. For each `#[ta_open_session]`, write down the TA's expected four-slot
   parameter layout. Use `ParametersNone` only when all four slots are expected
   to be `None`.
3. For each `#[ta_invoke_command]`, write down the TA's expected four-slot
   layout for each command ID.
4. Use a concrete 4-tuple when every command in the entry point uses the same
   layout.
5. Use `ParametersAny<'_>` when command IDs use different layouts.
6. Replace `as_value()?.a()` and `as_value()?.b()` with `get_a()` and
   `get_b()` on the correct value wrapper.
7. Replace `set_a()` and `set_b()` on old `ParamValue` with the same methods on
   `ParameterValueOutput` or `ParameterValueInout`.
8. Replace `as_memref()?.buffer()` reads with `get_buffer()`.
9. Replace manual output-buffer copies with `set_output`, `write_at`, or
   `get_buffer_mut` plus `set_updated_size`.
10. Ensure every unused slot is represented as `ParameterNone`, not omitted.

## Common Errors

### `TEE_ERROR_BAD_PARAMETERS` During Open Session

This means the runtime open-session parameters did not match the TA
open-session signature. If the TA expects any value or memref slot, do not use
`ParametersNone`; use a concrete tuple or `ParametersAny<'_>` that describes
the expected runtime layout.

### `TEE_ERROR_BAD_PARAMETERS` During Invoke Command

For `ParametersAny<'_>`, this usually means a branch called the wrong accessor:
for example `as_memref_input()` when the runtime slot is an output memref.

For a concrete tuple, the macro validates all four slots before entering the
function. Check the complete tuple, including the unused slots.

### Output Size Is Wrong for the Client Application

If TA code writes through `get_buffer_mut`, it must call `set_updated_size`.
Prefer `set_output` when possible because it copies and updates the size in one
step.

### Input/Output Direction Is Ambiguous

Choose the wrapper from the TA's perspective:

| Data flow | TA wrapper |
| --- | --- |
| Client application writes, TA reads | `ParameterMemrefInput<'_>` |
| TA writes, client application reads | `ParameterMemrefOutput<'_>` |
| Both read and write | `ParameterMemrefInout<'_>` |

## Legacy Compatibility

The deprecated API is still available as `optee_utee::deprecated`, and it
implements `FromRawParameters<'_>` for compatibility with the entry-point
macros. New and migrated code should use the typed wrappers instead.

Keeping the deprecated API in new code should be limited to transitional
patches, because it keeps the unsafe raw-pointer access pattern and does not
express parameter direction in the function signature.
