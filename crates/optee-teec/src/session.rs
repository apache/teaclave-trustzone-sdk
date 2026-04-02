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
use crate::{raw, Context, Error, Operation, Param, Result, Uuid};
use std::{cell::RefCell, ptr, rc::Rc};

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

/// Represents a connection between a client application and a trusted application.
pub struct Session {
    raw: raw::TEEC_Session,

    // Just a holder to ensure InnerContext is not dropped and to eliminate the
    // lifetime constraint, never use it.
    _ctx: Rc<RefCell<InnerContext>>,
}

// Since raw::TEEC_Session contains a raw pointer, Rust does not automatically
// implement Send and Sync for it. We need to manually implement them and ensure
// that raw::TEEC_Session is used safely.
unsafe impl Send for Session {}
unsafe impl Sync for Session {}

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
        let raw_operation = match operation {
            Some(o) => o.as_mut_raw_ptr(),
            None => ptr::null_mut(),
        };
        let inner_ctx = context.inner_context();
        let raw_ctx = &mut inner_ctx.borrow_mut().0;
        let raw_uuid = uuid.as_raw_ptr();

        match unsafe {
            raw::TEEC_OpenSession(
                raw_ctx,
                &mut raw_session,
                raw_uuid,
                login as u32,
                ptr::null(),
                raw_operation,
                &mut err_origin,
            )
        } {
            raw::TEEC_SUCCESS => Ok(Self {
                raw: raw_session,
                _ctx: context.inner_context(),
            }),
            code => Err(Error::from_raw_error(code).with_origin(err_origin.into())),
        }
    }

    /// Invokes a command with an operation with this session.
    pub fn invoke_command<A: Param, B: Param, C: Param, D: Param>(
        &mut self,
        command_id: u32,
        operation: &mut Operation<A, B, C, D>,
    ) -> Result<()> {
        let mut err_origin: u32 = 0;
        match unsafe {
            raw::TEEC_InvokeCommand(
                &mut self.raw,
                command_id,
                operation.as_mut_raw_ptr(),
                &mut err_origin,
            )
        } {
            raw::TEEC_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code).with_origin(err_origin.into())),
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            raw::TEEC_CloseSession(&mut self.raw);
        }
    }
}
