// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use super::context::InnerContext;
use crate::{Context, Operation, Param, Result, Uuid, raw, teec_check_with_origin};
use std::{
    ptr,
    sync::{Arc, Mutex},
};

/// Session login methods.
#[derive(Copy, Clone)]
pub enum ConnectionMethods {
    /// No login data is provided.
    LoginPublic,
    /// Login data about the user running the Client Application process is provided.
    LoginUser,
    /// Login data about the group running the Client Application process is provided.
    LoginGroup,
    /// Login data about the running Client Application itself is provided.
    LoginApplication,
    /// Login data about the user and the running Client Application itself is provided.
    LoginUserApplication,
    /// Login data about the group and the running Client Application itself is provided.
    LoginGroupApplication,
}

/// An opaque, thread-movable OP-TEE session handle.
///
/// `raw::TEEC_Session` mirrors a C struct containing a `*mut`, so Rust cannot
/// infer `Send`. libteec treats the handle as plain data: `TEEC_InvokeCommand`
/// and `TEEC_CloseSession` read `session->imp.session_id` / `session->imp.ctx`
/// and perform an ioctl, with no thread affinity (see `tee_client_api.c` in
/// optee_client). The handle is kept opaque here; the binding does not rely on
/// its layout.
struct RawTeecSession(raw::TEEC_Session);

// SAFETY: `RawTeecSession` is the sole owner of the handle and only ever hands
// it out as `&mut` (see `get_mut`), so callers cannot alias it. Moving it
// between threads is allowed by the libteec API contract: the handle is not
// thread-affine, and the owning `Context` (which owns the underlying fd) is
// kept alive separately by `Session::_ctx`. This assertion covers ownership
// transfer only; it does not imply `Sync`, and `Session` is deliberately not
// `Sync`.
unsafe impl Send for RawTeecSession {}

impl RawTeecSession {
    fn new(raw: raw::TEEC_Session) -> Self {
        Self(raw)
    }

    fn get_mut(&mut self) -> &mut raw::TEEC_Session {
        &mut self.0
    }
}

/// Represents a connection between a client application and a trusted application.
pub struct Session {
    raw: RawTeecSession,

    // Just a holder to ensure InnerContext is not dropped and to eliminate the
    // lifetime constraint, never use it.
    _ctx: Arc<Mutex<InnerContext>>,
}

impl Session {
    /// Initializes a TEE session object with specified context and uuid.
    pub fn new<A: Param, B: Param, C: Param, D: Param>(
        context: &mut Context,
        uuid: Uuid,
        login: ConnectionMethods,
        operation: Option<&mut Operation<A, B, C, D>>,
    ) -> Result<Self> {
        // SAFETY:
        // raw_session is a C struct(TEEC_Session), which zero value is valid.
        let mut raw_session = unsafe { std::mem::zeroed() };
        // define all parameters for raw::TEEC_OpenSession outside of the unsafe
        // block to maximize Rust's safety checks and leverage the compiler's
        // validation.
        let mut err_origin: u32 = 0;
        let raw_operation = operation.map_or(ptr::null_mut(), |o| o.as_mut_raw_ptr());
        let inner_ctx = context.inner_context();
        let mut guard = match inner_ctx.lock() {
            Ok(guard) => guard,
            // A poisoned lock only records that another thread panicked while
            // holding the context. The underlying `TEEC_Context` is still
            // usable, so recover it rather than failing this unrelated call.
            Err(poisoned) => poisoned.into_inner(),
        };
        let raw_ctx = &mut guard.0;
        let raw_uuid = uuid.as_raw_ptr();

        teec_check_with_origin(
            unsafe {
                raw::TEEC_OpenSession(
                    raw_ctx,
                    &mut raw_session,
                    raw_uuid,
                    login as u32,
                    ptr::null(),
                    raw_operation,
                    &mut err_origin,
                )
            },
            err_origin,
        )?;
        Ok(Self {
            raw: RawTeecSession::new(raw_session),
            _ctx: context.inner_context(),
        })
    }

    /// Invokes a command with an operation with this session.
    pub fn invoke_command<A: Param, B: Param, C: Param, D: Param>(
        &mut self,
        command_id: u32,
        operation: &mut Operation<A, B, C, D>,
    ) -> Result<()> {
        let mut err_origin: u32 = 0;
        teec_check_with_origin(
            unsafe {
                raw::TEEC_InvokeCommand(
                    self.raw.get_mut(),
                    command_id,
                    operation.as_mut_raw_ptr(),
                    &mut err_origin,
                )
            },
            err_origin,
        )
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            raw::TEEC_CloseSession(self.raw.get_mut());
        }
    }
}
