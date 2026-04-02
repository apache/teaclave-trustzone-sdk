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

/// Macro for printing to the trace output, without a newline.
///
/// Equivalent to the `trace_println!` macro expect that a newline is not
/// printed at the end of the message.
///
/// # Examples
///
/// ``` rust,no_run
/// # use optee_utee::trace_print;
///
/// trace_print!("this ");
/// trace_print!("will ");
/// trace_print!("be ");
/// trace_print!("on ");
/// trace_print!("the ");
/// trace_print!("same ");
/// trace_print!("line ");
/// trace_print!("this string has a newline, why not choose println! instead?\n");
/// ```
#[macro_export]
macro_rules! trace_print {
    ($($arg:tt)*) => ($crate::trace::Trace::_print(format_args!($($arg)*)));
}

/// Macro for printing to the trace output, with a newline.
/// Use the `format!` syntax to write data to the standard output. See
/// `std::fmt` for more information.
///
/// # Examples
///
/// ``` rust,no_run
/// # use optee_utee::trace_println;
/// trace_println!("Hello, World!");
/// trace_println!("format {} arguments", "some");
/// ```
#[macro_export]
macro_rules! trace_println {
    () => {
        $crate::trace::Trace::_print(format_args!("\n"));
    };
    ($s:expr) => {
        $crate::trace::Trace::_print(format_args!(concat!($s, "\n")));
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::trace::Trace::_print(format_args!(concat!($s, "\n"), $($tt)*));
    };
}
