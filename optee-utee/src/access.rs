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

//! Zero sized types and traits encoding access constraints into the type system.

mod private {
    pub trait Sealed {}
}

/// A type that is accessible (i.e. *not* [NoAccess])
pub trait Accessible: private::Sealed {}

/// A type that is readable
pub trait Readable: Accessible {}

/// A type that is writable
pub trait Writable: Accessible {}

/// Implements [`Accessible`], and [`Readable`]
#[derive(Debug, Default, Copy, Clone)]
pub struct Read;
impl private::Sealed for Read {}
impl Accessible for Read {}
impl Readable for Read {}

/// Implements [`Accessible`], and [`Writable`]
#[derive(Debug, Default, Copy, Clone)]
pub struct Write;
impl private::Sealed for Write {}
impl Accessible for Write {}
impl Writable for Write {}

/// Implements [`Accessible`], [`Readable`], [`Writable`]
#[derive(Debug, Default, Copy, Clone)]
pub struct ReadWrite;
impl private::Sealed for ReadWrite {}
impl Accessible for ReadWrite {}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

#[derive(Debug, Default, Copy, Clone)]
pub struct NoAccess;
impl private::Sealed for NoAccess {}
