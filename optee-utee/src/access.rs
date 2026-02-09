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
