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

use crate::{Error, Result, TeeParams, Uuid};
use optee_utee_sys as raw;

pub struct TaSessionBuilder<'a> {
    target_uuid: Uuid,
    timeout: u32,
    params: Option<TeeParams<'a>>,
}

impl<'a> TaSessionBuilder<'a> {
    /// Creates a new builder for the given TA UUID.
    pub fn new(uuid: Uuid) -> Self {
        Self {
            target_uuid: uuid,
            timeout: raw::TEE_TIMEOUT_INFINITE,
            params: None,
        }
    }

    /// Sets a custom timeout for the session opening.
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the parameters to be passed during session opening.
    pub fn with_params(mut self, params: TeeParams<'a>) -> Self {
        self.params = Some(params);
        self
    }

    /// Builds and opens the `TaSession`. Returns an error if the session fails to open.
    pub fn build(mut self) -> Result<TaSession> {
        let mut err_origin: u32 = 0;
        let mut raw_session: raw::TEE_TASessionHandle = core::ptr::null_mut();
        // Check if the parameters are provided and prepare them for the C API call.
        let (raw_param_types, raw_params_ptr, raw_params_opt) =
            if let Some(params) = &mut self.params {
                let mut raw_params = params.as_raw();
                let raw_ptr = raw_params.as_mut_ptr();
                (params.raw_param_types(), raw_ptr, Some(raw_params))
            } else {
                (0, core::ptr::null_mut(), None)
            };

        // SAFETY:
        // self.target_uuid.as_raw_ptr() provides a valid pointer to the UUID.
        // raw_params.as_mut_ptr() provides a valid pointer to the parameters.
        // The remaining arguments are either valid values or null/mut pointers as expected by the C API.
        // For parameters that are intended to be modified by the call, the buffer constraints are checked later in update_from_raw().
        match unsafe {
            raw::TEE_OpenTASession(
                self.target_uuid.as_raw_ptr(),
                self.timeout,
                raw_param_types,
                raw_params_ptr,
                &mut raw_session,
                &mut err_origin,
            )
        } {
            raw::TEE_SUCCESS => {
                if let (Some(params), Some(raw_params)) = (&mut self.params, raw_params_opt) {
                    params.update_from_raw(&raw_params)?;
                }

                Ok(TaSession { raw: raw_session })
            }
            code => Err(Error::from_raw_error(code).with_origin(err_origin.into())),
        }
    }
}

pub struct TaSession {
    raw: raw::TEE_TASessionHandle,
}

impl TaSession {
    /// Invokes a command with the provided parameters using the session's default timeout.
    /// Returns the result directly without allowing further method chaining.
    pub fn invoke_command(&mut self, command_id: u32, params: &mut TeeParams) -> Result<()> {
        self.invoke_command_with_timeout(command_id, params, raw::TEE_TIMEOUT_INFINITE)
    }

    pub fn invoke_command_with_timeout(
        &mut self,
        command_id: u32,
        params: &mut TeeParams,
        timeout: u32,
    ) -> Result<()> {
        let mut err_origin: u32 = 0;
        let mut raw_params = params.as_raw();
        let param_types = params.raw_param_types();

        // SAFETY:
        // self.raw is a valid pointer to an active session handle.
        // raw_params.as_mut_ptr() yields a valid mutable pointer to the parameters array.
        // The remaining arguments are either valid values or null/mutable pointers, as expected by the C API.
        // For parameters that are intended to be modified by the call, the buffer constraints are checked later in update_from_raw().
        match unsafe {
            raw::TEE_InvokeTACommand(
                self.raw,
                timeout,
                command_id,
                param_types,
                raw_params.as_mut_ptr(),
                &mut err_origin,
            )
        } {
            raw::TEE_SUCCESS => {
                // Update the parameters with the results
                params.update_from_raw(&raw_params)?;
                Ok(())
            }
            code => Err(Error::from_raw_error(code).with_origin(err_origin.into())),
        }
    }
}

// Drop implementation to close the session
impl Drop for TaSession {
    fn drop(&mut self) {
        // SAFETY:
        // self.raw is a valid pointer to an active session handle.
        // The function call is expected to clean up the session resources.
        unsafe {
            raw::TEE_CloseTASession(self.raw);
        }
    }
}
