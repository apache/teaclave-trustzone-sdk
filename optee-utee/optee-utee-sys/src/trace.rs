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

unsafe extern "C" {
    pub fn trace_ext_puts(str: *const c_char);
    pub fn trace_ext_get_thread_id() -> c_int;
    pub fn trace_set_level(level: c_int);
    pub fn trace_get_level() -> c_int;
    pub fn trace_printf(
        func: *const c_char,
        line: c_int,
        level: c_int,
        level_ok: bool,
        fmt: *const c_char,
        ...
    );
    pub fn dhex_dump(
        function: *const c_char,
        line: c_int,
        level: c_int,
        buf: *const c_void,
        len: c_int,
    );
}
