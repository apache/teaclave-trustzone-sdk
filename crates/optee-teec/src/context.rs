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

use crate::{raw, ConnectionMethods, Error, Operation, Param, ParamNone, Result, Session, Uuid};
use std::{cell::RefCell, ptr, rc::Rc};

pub struct InnerContext(pub raw::TEEC_Context);

impl Drop for InnerContext {
    fn drop(&mut self) {
        unsafe {
            raw::TEEC_FinalizeContext(&mut self.0);
        }
    }
}

/// An abstraction of the logical connection between a client application and a
/// TEE.
pub struct Context {
    // Use Rc to share it with Session, eliminating the lifetime constraint.
    // Use RefCell to allow conversion into a raw mutable pointer.
    // As RefCell is not Send + Sync, there is no need to use Arc.
    raw: Rc<RefCell<InnerContext>>,
}

// Since RefCell is used for Context, Rust does not automatically implement
// Send and Sync for it. We need to manually implement them and ensure that
// InnerContext is used correctly.
unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    /// Creates a TEE client context object.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// # use optee_teec::Context;
    /// # fn main() -> optee_teec::Result<()> {
    /// let mut ctx = Context::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Context> {
        // SAFETY:
        // raw_ctx is a C struct(TEEC_Context), which zero value is valid.
        let mut raw_ctx = unsafe { std::mem::zeroed() };
        match unsafe { raw::TEEC_InitializeContext(ptr::null_mut(), &mut raw_ctx) } {
            raw::TEEC_SUCCESS => Ok(Self {
                raw: Rc::new(RefCell::new(InnerContext(raw_ctx))),
            }),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Opens a new session with the specified trusted application.
    ///
    /// The target trusted application is specified by `uuid`.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use optee_teec::{Context, ErrorKind, Uuid};
    ///
    /// fn main() -> optee_teec::Result<()> {
    ///     let mut ctx = Context::new()?;
    ///     let uuid = Uuid::parse_str("8abcf200-2450-11e4-abe2-0002a5d5c51b").map_err(|err| {
    ///         println!("bad uuid: {:?}", err);
    ///         ErrorKind::BadParameters
    ///     })?;
    ///     let session = ctx.open_session(uuid)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open_session(&mut self, uuid: Uuid) -> Result<Session> {
        Session::new(
            self,
            uuid,
            ConnectionMethods::LoginPublic,
            None::<&mut Operation<ParamNone, ParamNone, ParamNone, ParamNone>>,
        )
    }

    pub fn open_session_with_login(
        &mut self,
        uuid: Uuid,
        login: ConnectionMethods,
    ) -> Result<Session> {
        Session::new(
            self,
            uuid,
            login,
            None::<&mut Operation<ParamNone, ParamNone, ParamNone, ParamNone>>,
        )
    }

    /// Opens a new session with the specified trusted application, pass some
    /// parameters to TA by an operation.
    ///
    /// The target trusted application is specified by `uuid`.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use optee_teec::{Context, ErrorKind, Operation, ParamNone, ParamType, ParamValue, Uuid};
    ///
    /// fn main() -> optee_teec::Result<()> {
    ///     let mut ctx = Context::new()?;
    ///     let uuid = Uuid::parse_str("8abcf200-2450-11e4-abe2-0002a5d5c51b").map_err(|err| {
    ///         println!("bad uuid: {:?}", err);
    ///         ErrorKind::BadParameters
    ///     })?;
    ///     let p0 = ParamValue::new(42, 0, ParamType::ValueInout);
    ///     let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    ///     let session = ctx.open_session_with_operation(uuid, &mut operation)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open_session_with_operation<A: Param, B: Param, C: Param, D: Param>(
        &mut self,
        uuid: Uuid,
        operation: &mut Operation<A, B, C, D>,
    ) -> Result<Session> {
        Session::new(self, uuid, ConnectionMethods::LoginPublic, Some(operation))
    }
}

// Internal usage only
impl Context {
    // anyone who wants to access the inner_context must take this as mut.
    pub(crate) fn inner_context(&mut self) -> Rc<RefCell<InnerContext>> {
        self.raw.clone()
    }
}
