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

#![cfg_attr(doc, doc = concat!(
    env!("CARGO_PKG_DESCRIPTION"),
    "\n",
    "## Feature flags\n",
    document_features::document_features!(),
))]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

// The bindings live in their own module so `handwritten` can refer to them as
// `generated::*`. `include!` keeps the file out of `cargo fmt`: bindgen formats
// the C doc comments its own way and rewriting them here would make every
// regeneration a formatting diff.
mod generated {
    include!("generated.rs");
}
mod handwritten;

pub use generated::*;
pub use handwritten::*;
