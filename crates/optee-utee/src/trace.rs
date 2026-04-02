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

use core::ffi::*;
use core::fmt::{Arguments, Result, Write};
use optee_utee_sys as raw;

pub struct Trace;

impl Trace {
    fn new() -> Self {
        Trace {}
    }

    pub fn _print(fmt: Arguments) {
        let mut writer = Trace::new();
        let result = writer.write_fmt(fmt);

        if let Err(e) = result {
            panic!("failed printing to trace: {}", e);
        }
    }

    pub fn set_level(level: i32) {
        unsafe {
            raw::trace_set_level(level);
        }
    }

    pub fn get_level() -> i32 {
        unsafe { raw::trace_get_level() }
    }
}

impl Write for Trace {
    fn write_str(&mut self, buf: &str) -> Result {
        unsafe {
            raw::_utee_log(buf.as_ptr() as *const c_void, buf.len());
        }
        Ok(())
    }
}
