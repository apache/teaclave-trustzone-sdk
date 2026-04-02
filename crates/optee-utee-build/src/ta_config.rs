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
use crate::Error;
use std::convert::TryInto;

/// Configuration options for TA
///
/// Examples
///
/// # use a default configuration
/// ```rust
/// use optee_utee_build::TaConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default(
///     UUID,
///     "0.1.0",
///     "hello world",
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// and since we already have `version` and `description` in `cargo.toml`,
/// we can make it simpler by using them as parameters:
///
/// ```rust
/// use optee_utee_build::TaConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default_with_cargo_env(UUID)?;
/// # Ok(())
/// # }
/// ```
///
/// # make some modifications
/// ```rust
/// use optee_utee_build::TaConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TaConfig::new_default(
///     UUID,
///     "0.1.0",
///     "hello world",
/// )?.ta_stack_size(10 * 1024).ta_data_size(32 * 1024);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TaConfig {
    pub uuid: uuid::Uuid,
    pub ta_flags: u32,
    pub ta_data_size: u32,
    pub ta_stack_size: u32,
    pub ta_version: String,
    pub ta_description: String,
    pub trace_level: i32,
    pub trace_ext_prefix: String,
    pub ta_framework_stack_size: u32,
    pub ext_properties: Vec<Property>,
}

impl TaConfig {
    /// Generate a default config by uuid, this is a wrapper of `new_default`
    /// function, it retrieves version and description from your TA's cargo.toml
    /// by environment-variables provided by cargo for building script, make
    /// constructor simpler.
    ///
    /// If your version and description of TA are different with the version and
    /// description of your crate, use `new_default` to provide them manually.
    pub fn new_default_with_cargo_env(uuid_str: &str) -> Result<Self, Error> {
        Self::new_default(
            uuid_str,
            std::env::var("CARGO_PKG_VERSION")?.as_str(),
            std::env::var("CARGO_PKG_DESCRIPTION")?.as_str(),
        )
    }
    /// generate a default config
    pub fn new_default(
        uuid_str: &str,
        ta_version: &str,
        ta_description: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            uuid: uuid_str.try_into()?,
            ta_flags: 0,
            ta_data_size: 32 * 1024,
            ta_stack_size: 2 * 1024,
            ta_version: ta_version.to_string(),
            ta_description: ta_description.to_string(),
            trace_level: 4,
            trace_ext_prefix: "TA".to_string(),
            ta_framework_stack_size: 2048,
            ext_properties: Vec::new(),
        })
    }
    pub fn ta_flags(mut self, flags: u32) -> Self {
        self.ta_flags = flags;
        self
    }
    pub fn ta_stack_size(mut self, stack_size: u32) -> Self {
        self.ta_stack_size = stack_size;
        self
    }
    pub fn ta_data_size(mut self, size: u32) -> Self {
        self.ta_data_size = size;
        self
    }
    pub fn trace_level(mut self, level: i32) -> Self {
        self.trace_level = level;
        self
    }
    pub fn trace_ext_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.trace_ext_prefix = prefix.into();
        self
    }
    pub fn ta_framework_stack_size(mut self, stack_size: u32) -> Self {
        self.ta_framework_stack_size = stack_size;
        self
    }
    pub fn add_ext_property(mut self, name: &str, value: PropertyValue) -> Self {
        self.ext_properties.push(Property::new(name, value));
        self
    }
}

/// An enum of PropertyValue, with its type and value combined
///
/// Usage:
/// ```rust
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// # use optee_utee_build::PropertyValue;
/// # use std::convert::TryInto;
/// # const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// # const LOGIN: u32 = 1;
/// PropertyValue::Bool(true);
/// PropertyValue::U32(1);
/// PropertyValue::Uuid(UUID.try_into()?);
/// PropertyValue::Identity(LOGIN, UUID.try_into()?);
/// // a string value, must not append '\0' at last, we will add it for you.
/// PropertyValue::Str("hello world".to_string());
/// // a base64 string value, must not append '\0' at last, we will add it for you.
/// PropertyValue::BinaryBlock("c2RmYXNm".to_string());
/// PropertyValue::U64(1);
/// # Ok(())
/// # }
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Bool(bool),
    U32(u32),
    Uuid(uuid::Uuid),
    Identity(u32, uuid::Uuid),
    Str(String),
    BinaryBlock(String),
    U64(u64),
}

/// A GP property pair, use it to set ta_properties
///
/// must not append a '\0' in name, we will add it automatically if neccessary.
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    /// value of the property
    pub value: PropertyValue,
}

impl Property {
    pub fn new(name: &str, value: PropertyValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}
