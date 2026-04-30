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

use crate::{
    raw::size_t,
    {ErrorKind, Result},
};
use core::ffi::c_void;

/// Parameters for a plugin invocation, carrying the command, sub-command,
/// and the inout buffer.
///
/// The core design goal of this struct is to prevent developers from forgetting
/// to set `out_len`. In the C ABI, `out_len` is a raw pointer that the plugin
/// must write to in order to report how many bytes it actually
/// produced. Forgetting to set it is a silent bug — the TA caller receives
/// garbage (uninitialized or stale) length, leading to buffer over-reads or
/// truncated output that is extremely hard to diagnose.
///
/// To eliminate this class of bugs, `PluginParameters` ties `out_len` to
/// every output-writing operation: `write_output_at` and `set_buf_from_slice`,
/// both automatically update `out_len` on success, so the plugin developers
/// never has to do it manually. If plugin developers need full control, they
/// can use `get_buffer_mut` and `set_out_len` explicitly.
pub struct PluginParameters<'a, 'b> {
    /// Command identifier for the plugin invocation.
    pub cmd: u32,
    /// Sub-command identifier for the plugin invocation.
    pub sub_cmd: u32,
    /// Inout buffer that carries input data into the plugin and receives
    /// output data from it.
    buf: &'a mut [u8],
    /// Pointer to the output length that the plugin must set.
    /// Wrapped in `Option` because the C API allows it to be NULL
    /// (meaning the caller does not expect output). When present,
    /// every output-writing method automatically updates this value,
    /// ensuring `out_len` is never left unset.
    out_len: Option<&'b mut size_t>,
}

impl<'a, 'b> PluginParameters<'a, 'b> {
    /// Constructs a `PluginParameters` from raw C pointers.
    ///
    /// # Safety
    /// - `buf` must be valid for reads/writes of `in_len` bytes if not null
    /// - `out_len` must be valid for writes if not null
    /// - both pointers must remain alive for the lifetime of the returned
    ///   `PluginParameters`
    ///
    /// When `out_len` is non-null, it will be tracked by the returned struct
    /// so that output-writing methods can update it automatically — this is
    /// the key mechanism that prevents the "forgot to set out_len" bug.
    pub unsafe fn from_raw(
        cmd: u32,
        sub_cmd: u32,
        buf: *mut c_void,
        in_len: size_t,
        out_len: *mut size_t,
    ) -> Result<Self> {
        // Reject obviously invalid parameter combinations:
        // a non-zero in_len or a present out_len implies the caller expects
        // to use the buffer, so a null buf pointer is illegal.
        if (in_len != 0 || !out_len.is_null()) && buf.is_null() {
            return Err(ErrorKind::BadParameters.into());
        }
        // Wrap the raw buffer pointer into a safe slice.
        // For current OP-TEE, buf should always be non-null.
        let buf = match buf.is_null() {
            true => &mut [],
            false => unsafe { core::slice::from_raw_parts_mut(buf as *mut _, in_len) },
        };
        // Track the out_len pointer so output-writing methods can update it
        // automatically — this is what prevents "forgot to set out_len" bugs.
        let out_len = unsafe { out_len.as_mut() };

        Ok(Self {
            cmd,
            sub_cmd,
            buf,
            out_len,
        })
    }

    /// Copies the entire `sendslice` into the inout buffer starting at offset
    /// 0, and automatically sets `out_len` to `sendslice.len()`.
    ///
    /// This is the primary safe way to write output — callers do not need to
    /// update `out_len` separately.
    ///
    /// Returns `ShortBuffer` if the buffer is too small, or `BadState` if
    /// the output length pointer is not available.
    pub fn set_buf_from_slice(&mut self, sendslice: &[u8]) -> Result<()> {
        self.write_output_at(0, sendslice)
    }

    /// Writes `data` into the inout buffer at the given `pos`, and
    /// automatically updates `out_len` to `pos + data.len()`.
    ///
    /// By always updating `out_len` on a successful write, this method
    /// eliminates the risk of the developer forgetting to set it.
    ///
    /// Returns `ShortBuffer` if the buffer is too small, or `BadState` if
    /// the output length pointer is not available.
    pub fn write_output_at(&mut self, pos: usize, data: &[u8]) -> Result<()> {
        if let Some(out_len) = self.out_len.as_mut() {
            let dest_len = pos + data.len();
            if self.buf.len() < dest_len {
                // Buffer overflow: not enough space for the write
                log::debug!("Overflow: Input length is less than output length");
                return Err(ErrorKind::ShortBuffer.into());
            }
            self.buf[pos..dest_len].copy_from_slice(data);
            (**out_len) = dest_len;
            return Ok(());
        }
        log::debug!("output is not allowed");
        Err(ErrorKind::BadState.into())
    }

    /// Returns a shared reference to the inout buffer.
    pub fn get_buffer(&self) -> &[u8] {
        self.buf
    }

    /// Returns a mutable reference to the inout buffer.
    ///
    /// # Safety
    /// The caller is responsible for updating `out_len` (via [`set_out_len`])
    /// after writing to the buffer.
    pub unsafe fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.buf
    }

    /// Explicitly sets `out_len` to the given value.
    ///
    /// This is an escape hatch for cases where the caller needs full control
    /// over the output length (e.g. after using `get_buffer_mut`). In most
    /// cases should prefer `write_output_at` or `set_buf_from_slice`, which set
    /// `out_len` automatically and avoid the "forgot to set out_len" bug.
    ///
    /// Returns `BadParameters` if `out_len` exceeds the buffer size, or
    /// `BadState` if the output length pointer is not available.
    pub fn set_out_len(&mut self, out_len: usize) -> Result<()> {
        if out_len > self.buf.len() {
            return Err(ErrorKind::BadParameters.into());
        }
        match self.out_len.as_mut() {
            None => Err(ErrorKind::BadState.into()),
            Some(v) => {
                **v = out_len;
                Ok(())
            }
        }
    }
}
