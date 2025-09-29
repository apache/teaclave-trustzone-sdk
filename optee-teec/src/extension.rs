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

use crate::raw;
use crate::{Error, ErrorKind, Result};
use core::ffi::c_char;

#[repr(C)]
pub struct PluginMethod {
    pub name: *const c_char,
    pub uuid: raw::TEEC_UUID,
    pub init: fn() -> raw::TEEC_Result,
    pub invoke: unsafe fn(
        cmd: u32,
        sub_cmd: u32,
        data: *mut c_char,
        in_len: u32,
        out_len: *mut u32,
    ) -> raw::TEEC_Result,
}

/// struct PluginParameters {
/// @cmd: u32,              plugin cmd, defined in proto/
/// @sub_cmd: u32,          plugin subcmd, defined in proto/
/// @inout: &'a mut [u8],   input/output buffer shared with TA and plugin
/// @required_outlen,                length of output sent to TA
/// }
pub struct PluginParameters<'a> {
    pub cmd: u32,
    pub sub_cmd: u32,
    pub inout: &'a mut [u8],
    required_outlen: usize,
}
impl<'a> PluginParameters<'a> {
    pub fn new(cmd: u32, sub_cmd: u32, inout: &'a mut [u8]) -> Self {
        Self {
            cmd,
            sub_cmd,
            inout,
            required_outlen: 0_usize,
        }
    }

    // This function copies data from the provided slice to the inout buffer
    // Please note that the Result should be properly handled by the caller
    // If the inout buffer is smaller than the provided slice, an error will be returned
    // Ignoring this error will lead to undefined behavior
    pub fn set_buf_from_slice(&mut self, sendslice: &[u8]) -> Result<()> {
        if self.inout.len() < sendslice.len() {
            println!("Overflow: Input length is less than output length");
            self.required_outlen = sendslice.len();
            return Err(Error::new(ErrorKind::ShortBuffer));
        }
        self.required_outlen = sendslice.len();
        self.inout[..self.required_outlen].copy_from_slice(sendslice);
        Ok(())
    }

    /// This function returns the required output length
    /// If the inout buffer is too small, this indicates the size needed
    /// If the inout buffer is large enough, this is the actual output length
    pub fn get_required_out_len(&self) -> usize {
        self.required_outlen
    }
}
