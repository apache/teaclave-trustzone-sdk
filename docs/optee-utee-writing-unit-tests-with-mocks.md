---
permalink: /trustzone-sdk-docs/optee-utee-writing-unit-tests-with-mocks
---

# Writing Unit Tests with Mocks for OP-TEE Modules

This guide explains how to write unit tests in `optee-utee` crate using the
built-in `mock` feature of `optee-utee-sys` crate.

The `mock` feature is built on the [`mockall`](https://crates.io/crates/mockall)
crate. It automatically generates mock implementations of the OP-TEE Internal
Core API.

## Running Mock-Based Tests

Mock tests run on the host machine using standard `cargo test`:

```bash
cd crates
cargo test -p optee-utee --features no_panic_handler -vv
```

The `no_panic_handler` feature prevents the custom panic handler from
interfering with test execution.

## Writing Your First Mock Test

### Basic Test Structure

All mock tests follow a consistent pattern:

```rust
#[cfg(test)]
mod tests {
    // Required: import std into no_std crate for testing
    extern crate std;

    use optee_utee_sys::{
        mock_api,
        mock_utils::SERIAL_TEST_LOCK,
    };
    use super::*;

    #[test]
    fn test_my_function() {
        // 1. Acquire the serial test lock
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        // 2. Get mock context for the API you want to mock
        let ctx = mock_api::TEE_SomeFunction_context();

        // 3. Set up expectations
        ctx.expect().return_once_st(|params| {
            // Return success or specific error code
            raw::TEE_SUCCESS
        });

        // 4. Execute code under test
        let result = my_function();

        // 5. Assert results
        assert!(result.is_ok());
    }
}
```

### Key Imports

```rust
use optee_utee_sys::{
    mock_api,                                    // Mock API contexts
    mock_utils::SERIAL_TEST_LOCK,                // Global test lock
    mock_utils::object::MockHandle,              // Mock object handles
};
use optee_utee_sys as raw;                       // For TEE_SUCCESS, etc.
```

## Mock Expectation Methods

The `mockall` crate provides several methods for setting up mock behavior:

| Method | Description | Use Case |
|--------|-------------|----------|
| `.return_once_st(value)` | Returns `value` exactly once | Single API call |
| `.returning_st(closure)` | Calls closure for each invocation | Multiple calls with dynamic behavior |
| `.return_const_st(value)` | Always returns constant value | Simple stubbing |
| `.times(n)` | Expects exactly `n` calls | Verify call count |

The `_st` suffix stands for "single-threaded" — required because mockall's
default methods use thread-local storage incompatible with this test setup.

## Common Patterns

### Pattern 1: Testing Success Cases

```rust
#[test]
fn test_open_success() {
    let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

    let mut raw_handle = MockHandle::new();
    let handle = raw_handle.as_handle();

    let ctx = mock_api::TEE_OpenPersistentObject_context();
    ctx.expect()
        .return_once_st(move |_, _, _, _, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });

    let result = PersistentObject::open(
        ObjectStorageConstants::Private,
        &[0x01],
        DataFlag::ACCESS_READ,
    );

    assert!(result.is_ok());
}
```

### Pattern 2: Testing Error Cases

```rust
#[test]
fn test_open_not_found() {
    let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

    static RETURN_CODE: raw::TEE_Result = raw::TEE_ERROR_ITEM_NOT_FOUND;
    let ctx = mock_api::TEE_OpenPersistentObject_context();
    ctx.expect().return_const_st(RETURN_CODE);

    let result = PersistentObject::open(
        ObjectStorageConstants::Private,
        &[0x01],
        DataFlag::ACCESS_READ,
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().raw_code(), RETURN_CODE);
}
```

### Pattern 3: Testing Drop Behavior

```rust
#[test]
fn test_create_and_drop() {
    let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

    let mut raw_handle = MockHandle::new();
    let handle = raw_handle.as_handle();

    let create_ctx = mock_api::TEE_CreatePersistentObject_context();
    let close_ctx = mock_api::TEE_CloseObject_context();

    create_ctx.expect()
        .return_once_st(move |_, _, _, _, _, _, _, obj| {
            unsafe { *obj = handle.clone() };
            raw::TEE_SUCCESS
        });

    close_ctx.expect().return_once_st(move |obj| {
        debug_assert_eq!(obj, handle);
    });

    {
        let _obj = PersistentObject::create(
            ObjectStorageConstants::Private,
            &[],
            DataFlag::ACCESS_WRITE,
            None,
            &[],
        ).expect("should succeed");
        // _obj dropped here, triggering TEE_CloseObject
    }
}
```

### Pattern 4: Multiple API Calls

Use `.times(n)` for checking APIs called multiple times:

```rust
use std::sync::{Arc, Mutex};

#[test]
fn test_multiple_calls() {
    let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

    let call_count = Arc::new(Mutex::new(0));

    let ctx = mock_api::TEE_SomeApi_context();
    ctx.expect()
        .returning_st({
            let call_count = call_count.clone();
            move |_| {
                let mut count = call_count.lock().unwrap();
                *count += 1;
            }
        })
        .times(3);  // Expect exactly 3 calls

    // ... execute code that calls the API 3 times

    assert_eq!(*call_count.lock().unwrap(), 3);
}
```

### Pattern 5: Buffer Manipulation

For APIs that read/write through pointers, simulate buffer operations:

```rust
#[test]
fn test_read_data() {
    let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

    let expected_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let ctx = mock_api::TEE_ReadObjectData_context();

    ctx.expect()
        .return_once_st(move |_, buf, size, count| {
            let buffer: &mut [u8] = unsafe {
                core::slice::from_raw_parts_mut(buf as *mut u8, size)
            };
            let len = expected_data.len().min(size);
            buffer[..len].copy_from_slice(&expected_data[..len]);
            unsafe { *count = len };
            raw::TEE_SUCCESS
        });

    // ... execute read operation
}
```

## Mocking Object Handles

Use `MockHandle` to create mock object handles for testing:

```rust
let mut raw_handle = MockHandle::new();
let handle = raw_handle.as_handle();  // Returns TEE_ObjectHandle

// Pass `handle` to mock expectations that return or compare object handles
```

## The Serial Test Lock

**Always acquire `SERIAL_TEST_LOCK` at the start of every mock test:**

```rust
let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");
```

This is **critical** because mockall's mock contexts use global state.
Without this lock, concurrent tests would interfere with each other's
expectations, causing flaky test failures.


## Examples

Study these existing test implementations for more patterns:

| File | Demonstrates |
|------|-------------|
| `crates/optee-utee/src/extension.rs` | Plugin invocation with buffer manipulation |
| `crates/optee-utee/src/object/persistent_object.rs` | Create/open/drop lifecycle, error cases |
| `crates/optee-utee/src/object/transient_object.rs` | Transient object allocation and freeing |
| `crates/optee-utee/src/object/object_handle.rs` | Handle validation and close behavior |

## Limitations

- **Mock tests only verify API call patterns**, not actual cryptographic operations or hardware behavior
- **Tests must run sequentially** — always use `SERIAL_TEST_LOCK`
