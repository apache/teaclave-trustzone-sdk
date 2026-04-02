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

use crate::{Error, ErrorKind, ParamType, Result};
use core::ops::{Index, IndexMut};
use optee_utee_sys as raw;

#[derive(Copy, Clone, Debug)]
pub enum ParamIndex {
    Arg0,
    Arg1,
    Arg2,
    Arg3,
}

impl ParamIndex {
    fn to_usize(self) -> usize {
        match self {
            ParamIndex::Arg0 => 0,
            ParamIndex::Arg1 => 1,
            ParamIndex::Arg2 => 2,
            ParamIndex::Arg3 => 3,
        }
    }
}

enum ParamContent<'a> {
    None,
    MemrefInput {
        buffer: &'a [u8],
    },
    MemrefOutput {
        buffer: &'a mut [u8],
        written: usize,
    },
    MemrefInout {
        buffer: &'a mut [u8],
        written: usize,
    },
    ValueInput {
        a: u32,
        b: u32,
    },
    ValueOutput {
        a: u32,
        b: u32,
    },
    ValueInout {
        a: u32,
        b: u32,
    },
}

pub struct Param<'a> {
    content: ParamContent<'a>,
}

impl<'a> Param<'a> {
    fn new() -> Self {
        Self {
            content: ParamContent::None,
        }
    }

    /// Returns the written slice if available (for `MemrefOutput` or `MemrefInout`).
    /// Returns `None` for other types. Developers can decide whether to treat absence as an error.
    pub fn written_slice(&self) -> Option<&[u8]> {
        match &self.content {
            ParamContent::MemrefOutput { buffer, written } => Some(&buffer[..*written]),
            ParamContent::MemrefInout { buffer, written } => Some(&buffer[..*written]),
            _ => None,
        }
    }

    /// Returns the output values if available (for `ValueOutput` or `ValueInout`).
    /// Returns `None` for other types. Caller decides how to handle missing values.
    pub fn output_value(&self) -> Option<(u32, u32)> {
        match &self.content {
            ParamContent::ValueOutput { a, b } => Some((*a, *b)),
            ParamContent::ValueInout { a, b } => Some((*a, *b)),
            _ => None,
        }
    }

    fn get_type(&self) -> ParamType {
        match &self.content {
            ParamContent::None => ParamType::None,
            ParamContent::MemrefInput { .. } => ParamType::MemrefInput,
            ParamContent::MemrefOutput { .. } => ParamType::MemrefOutput,
            ParamContent::MemrefInout { .. } => ParamType::MemrefInout,
            ParamContent::ValueInput { .. } => ParamType::ValueInput,
            ParamContent::ValueOutput { .. } => ParamType::ValueOutput,
            ParamContent::ValueInout { .. } => ParamType::ValueInout,
        }
    }

    fn get_raw_type(&self) -> u32 {
        self.get_type() as u32
    }

    fn as_raw(&mut self) -> raw::TEE_Param {
        match &mut self.content {
            ParamContent::None => raw::TEE_Param {
                memref: raw::Memref {
                    buffer: core::ptr::null_mut(),
                    size: 0,
                },
            },
            ParamContent::MemrefInput { buffer } => raw::TEE_Param {
                memref: raw::Memref {
                    buffer: (*buffer).as_ptr() as *mut core::ffi::c_void,
                    size: buffer.len(),
                },
            },
            ParamContent::MemrefOutput { buffer, written: _ } => raw::TEE_Param {
                memref: raw::Memref {
                    buffer: (*buffer).as_mut_ptr() as *mut core::ffi::c_void,
                    size: buffer.len(),
                },
            },
            ParamContent::MemrefInout { buffer, written: _ } => raw::TEE_Param {
                memref: raw::Memref {
                    buffer: (*buffer).as_mut_ptr() as *mut core::ffi::c_void,
                    size: buffer.len(),
                },
            },
            ParamContent::ValueInput { a, b } => raw::TEE_Param {
                value: raw::Value { a: *a, b: *b },
            },
            ParamContent::ValueInout { a, b } => raw::TEE_Param {
                value: raw::Value { a: *a, b: *b },
            },
            ParamContent::ValueOutput { a, b } => raw::TEE_Param {
                value: raw::Value { a: *a, b: *b },
            },
        }
    }

    fn update_size_from_raw(&mut self, raw_param: &raw::TEE_Param) -> Result<()> {
        match &mut self.content {
            ParamContent::MemrefOutput { buffer, written } => {
                // SAFETY:
                // The caller must ensure this param is of memref type and properly initialized.
                // This is enforced by the variant match on `ParamContent::MemrefOutput`.
                // Accessing `raw_param.memref.size` is safe under these assumptions.
                let new_size = unsafe { raw_param.memref.size };
                if new_size > (*buffer).len() {
                    return Err(Error::new(ErrorKind::BadParameters));
                }
                *written = new_size;
                Ok(())
            }
            ParamContent::MemrefInout { buffer, written } => {
                // SAFETY:
                // The caller must ensure this param is of memref type and properly initialized.
                // This is enforced by the variant match on `ParamContent::MemrefOutput`.
                // Accessing `raw_param.memref.size` is safe under these assumptions.
                let new_size = unsafe { raw_param.memref.size };
                if new_size > (*buffer).len() {
                    return Err(Error::new(ErrorKind::BadParameters));
                }
                *written = new_size;
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::BadFormat)),
        }
    }

    fn update_value_from_raw(&mut self, raw_param: &raw::TEE_Param) {
        match &mut self.content {
            ParamContent::ValueInout { a, b } => {
                // SAFETY:
                // The caller must ensure this param is of value type and properly initialized.
                // This is guaranteed by matching against `ParamContent::ValueInout`.
                // Accessing `raw_param.value.a` is safe under above assumption.
                *a = unsafe { raw_param.value.a };
                // SAFETY:
                // Accessing `raw_param.value.b` is safe under above assumption.
                *b = unsafe { raw_param.value.b };
            }
            ParamContent::ValueOutput { a, b } => {
                // SAFETY:
                // The caller must ensure this param is of value type and properly initialized.
                // This is guaranteed by matching against `ParamContent::ValueInout`.
                // Accessing `raw_param.value.a` is safe under above assumption.
                *a = unsafe { raw_param.value.a };
                // SAFETY:
                // Accessing `raw_param.value.b` is safe under above assumption.
                *b = unsafe { raw_param.value.b };
            }
            _ => {}
        }
    }
}

/// The TeeParams struct is used to manage the parameters for TEE commands.
pub struct TeeParams<'a> {
    params: [Param<'a>; 4],
}

impl<'a> Default for TeeParams<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TeeParams<'a> {
    pub fn new() -> Self {
        Self {
            params: [Param::new(), Param::new(), Param::new(), Param::new()],
        }
    }

    /// These functions allow for method-chaining to easily configure multiple parameters at once.
    ///
    /// The following methods can be chained:
    /// - `with_memref_in`: Sets a memory reference input parameter.
    /// - `with_memref_out`: Sets a memory reference output parameter.
    /// - `with_memref_inout`: Sets a memory reference inout parameter.
    /// - `with_value_in`: Sets a value input parameter.
    /// - `with_value_out`: Sets a value output parameter.
    /// - `with_value_inout`: Sets a value inout parameter.
    ///
    /// Example usage:
    /// ``` rust,no_run
    /// # use optee_utee::{TeeParams, ParamIndex};
    /// # let input_buffer = [0_u8; 0];
    /// # let mut output_buffer = [0_u8; 0];
    ///
    /// let params = TeeParams::new()
    ///     .with_memref_in(ParamIndex::Arg0, &input_buffer)
    ///     .with_memref_out(ParamIndex::Arg1, &mut output_buffer)
    ///     .with_value_in(ParamIndex::Arg2, 42, 0)
    ///     .with_value_out(ParamIndex::Arg3, 0, 0);
    /// ```
    pub fn with_memref_in(mut self, idx: ParamIndex, buffer: &'a [u8]) -> Self {
        self[idx].content = ParamContent::MemrefInput { buffer };
        self
    }

    pub fn with_memref_out(mut self, idx: ParamIndex, buffer: &'a mut [u8]) -> Self {
        self[idx].content = ParamContent::MemrefOutput { buffer, written: 0 };
        self
    }

    pub fn with_memref_inout(mut self, idx: ParamIndex, buffer: &'a mut [u8]) -> Self {
        self[idx].content = ParamContent::MemrefInout { buffer, written: 0 };
        self
    }

    pub fn with_value_in(mut self, idx: ParamIndex, a: u32, b: u32) -> Self {
        self[idx].content = ParamContent::ValueInput { a, b };
        self
    }

    pub fn with_value_out(mut self, idx: ParamIndex, a: u32, b: u32) -> Self {
        self[idx].content = ParamContent::ValueOutput { a, b };
        self
    }

    pub fn with_value_inout(mut self, idx: ParamIndex, a: u32, b: u32) -> Self {
        self[idx].content = ParamContent::ValueInout { a, b };
        self
    }

    /// These methods allow the user to set the content at a specific index.
    ///
    /// Example usage:
    /// ``` rust,no_run
    /// # use optee_utee::{TeeParams, ParamIndex};
    /// # let input_buffer = [0_u8; 0];
    /// # let mut output_buffer = [0_u8; 0];
    ///
    /// let mut params = TeeParams::new();
    /// params.set_memref_in(ParamIndex::Arg0, &input_buffer);
    /// params.set_memref_out(ParamIndex::Arg1, &mut output_buffer);
    /// params.set_value_in(ParamIndex::Arg2, 42, 0);
    /// params.set_value_out(ParamIndex::Arg3, 0, 0);
    /// ```
    pub fn set_memref_in(&mut self, idx: ParamIndex, buffer: &'a [u8]) -> &mut Self {
        self[idx].content = ParamContent::MemrefInput { buffer };
        self
    }

    pub fn set_memref_out(&mut self, idx: ParamIndex, buffer: &'a mut [u8]) -> &mut Self {
        self[idx].content = ParamContent::MemrefOutput { buffer, written: 0 };
        self
    }

    pub fn set_memref_inout(&mut self, idx: ParamIndex, buffer: &'a mut [u8]) -> &mut Self {
        self[idx].content = ParamContent::MemrefInout { buffer, written: 0 };
        self
    }

    pub fn set_value_in(&mut self, idx: ParamIndex, a: u32, b: u32) -> &mut Self {
        self[idx].content = ParamContent::ValueInput { a, b };
        self
    }

    pub fn set_value_out(&mut self, idx: ParamIndex, a: u32, b: u32) -> &mut Self {
        self[idx].content = ParamContent::ValueOutput { a, b };
        self
    }

    pub fn set_value_inout(&mut self, idx: ParamIndex, a: u32, b: u32) -> &mut Self {
        self[idx].content = ParamContent::ValueInout { a, b };
        self
    }

    pub(crate) fn raw_param_types(&self) -> u32 {
        let mut param_types = 0;
        for (i, param) in self.params.iter().enumerate() {
            param_types |= param.get_raw_type() << (i * 4);
        }
        param_types
    }

    pub(crate) fn as_raw(&mut self) -> [raw::TEE_Param; 4] {
        [
            self.params[0].as_raw(),
            self.params[1].as_raw(),
            self.params[2].as_raw(),
            self.params[3].as_raw(),
        ]
    }

    /// Updates the parameters with results after each TEE call.
    ///
    /// This function updates the content of parameters for `MemrefInout`, `MemrefOutput`, `ValueInout`, and `ValueOutput`.
    /// Parameters of other types are not modified.
    pub(crate) fn update_from_raw(&mut self, raw_params: &[raw::TEE_Param; 4]) -> Result<()> {
        // update the content for memref inout/out, and value inout/out
        for (i, param) in self.params.iter_mut().enumerate() {
            let raw_param = &raw_params[i];
            match param.get_type() {
                ParamType::MemrefOutput => {
                    param.update_size_from_raw(raw_param)?;
                }
                ParamType::MemrefInout => {
                    param.update_size_from_raw(raw_param)?;
                }
                ParamType::ValueOutput => {
                    param.update_value_from_raw(raw_param);
                }
                ParamType::ValueInout => {
                    param.update_value_from_raw(raw_param);
                }
                _ => {
                    // No action needed for other types
                }
            }
        }
        Ok(())
    }
}

// Index trait implementations for direct parameter access
impl<'a> Index<ParamIndex> for TeeParams<'a> {
    type Output = Param<'a>;

    fn index(&self, index: ParamIndex) -> &Self::Output {
        &self.params[index.to_usize()]
    }
}

impl<'a> IndexMut<ParamIndex> for TeeParams<'a> {
    fn index_mut(&mut self, index: ParamIndex) -> &mut Self::Output {
        &mut self.params[index.to_usize()]
    }
}
