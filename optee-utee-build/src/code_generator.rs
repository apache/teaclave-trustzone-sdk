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
use crate::{PropertyValue, TaConfig};
use quote::{format_ident, quote};
use std::str::FromStr;

/// Start from rust edition 2024, `no_mangle` and `link_section` must be wrapped with unsafe
#[derive(Clone)]
pub enum RustEdition {
    Before2024,
    Edition2024,
}

/// Generator of head file, use it to generate a header file and then include it in user codes.
///
/// Use only if you just want to generate a header file, and do the linking job yourself,
/// or you should use Builder instead.
///
/// Examples:
/// ```rust
/// use optee_utee_build::{HeaderFileGenerator, TaConfig, RustEdition};
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "26509cec-4a2b-4935-87ab-762d89fbf0b0";
/// let ta_config = TaConfig::new_default(UUID, "0.1.0", "example")?;
/// let codes = HeaderFileGenerator::new(RustEdition::Before2024).generate(&ta_config)?;
/// # Ok(())
/// # }
/// ```
pub struct HeaderFileGenerator {
    code: proc_macro2::TokenStream,
    edition: RustEdition,
}

impl HeaderFileGenerator {
    pub fn new(edition: RustEdition) -> Self {
        Self {
            code: quote!(),
            edition,
        }
    }
    pub fn generate(mut self, conf: &TaConfig) -> Result<String, Error> {
        self.write_includes();
        self.write_configurations(conf);
        self.write_trace(conf);
        self.write_properties(conf)?;
        self.write_ta_head(conf)?;
        self.write_ta_heap();

        const LICENSE_STR: &str = include_str!("./license_str.txt");
        let f = syn::parse2(self.code).unwrap();
        // prettyplease will remove all of the comments in it, and the
        // maintainer will not support keeping comments,
        // so we just add the comments to codes after formatting
        let code_string = format!("{}\n{}", LICENSE_STR, prettyplease::unparse(&f));
        Ok(code_string)
    }
}

impl HeaderFileGenerator {
    fn write_includes(&mut self) {
        self.code.extend(quote! {
        use core::ffi::*;
        use core::mem;
        use core::primitive::u64;
                });
    }

    fn write_trace(&mut self, conf: &TaConfig) {
        let trace_ext = string_to_binary_codes(&conf.trace_ext_prefix);
        let trace_level = conf.trace_level;
        let no_mangle_attribute = self.edition.no_mangle_attribute_codes();
        self.code.extend(quote! {
        #no_mangle_attribute
        pub static mut trace_level: c_int = #trace_level;

        #no_mangle_attribute
        pub static trace_ext_prefix: &[u8] = #trace_ext;

        /// # Safety
        /// This function is called by the OP-TEE framework to get the trace level.
        /// It's safe to call as it only reads a static variable.
        #no_mangle_attribute
        pub unsafe extern "C" fn tahead_get_trace_level() -> c_int {
            unsafe { trace_level }
        }
                })
    }
    fn write_configurations(&mut self, conf: &TaConfig) {
        let ta_version = string_to_binary_codes(&conf.ta_version);
        let ta_description = string_to_binary_codes(&conf.ta_description);
        let ta_flags = conf.ta_flags;
        let ta_data_size = conf.ta_data_size;
        let ta_stack_size = conf.ta_stack_size;
        self.code.extend(quote! {
        const TA_FLAGS: u32 = #ta_flags;
        const TA_DATA_SIZE: u32 = #ta_data_size;
        const TA_STACK_SIZE: u32 = #ta_stack_size;
        const TA_VERSION: &[u8] = #ta_version;
        const TA_DESCRIPTION: &[u8] = #ta_description;
                });
    }
    fn write_properties(&mut self, conf: &TaConfig) -> Result<(), Error> {
        let mut ext_property_codes =
            Vec::<proc_macro2::TokenStream>::with_capacity(conf.ext_properties.len());
        for (index, prop) in conf.ext_properties.iter().enumerate() {
            let var_name = format!("EXT_PROP_VALUE_{}", index + 1);
            let rust_type_name_codes = property_value_rust_type_declaration_codes(&prop.value);
            let value_codes = property_value_data_codes(&prop.value)?;
            let var_name_codes = format_ident!("{}", var_name);
            self.code.extend(quote! {
            const #var_name_codes: #rust_type_name_codes = #value_codes;
                        });
            let prop_name_codes = string_to_binary_codes(&prop.name);
            let utee_type_name_codes = property_value_utee_type_codes(&prop.value);
            let utee_value_conv_codes = property_value_as_utee_value_codes(&var_name, &prop.value);
            ext_property_codes.push(quote! {
                optee_utee_sys::user_ta_property {
                    name: #prop_name_codes.as_ptr(),
                    prop_type: #utee_type_name_codes,
                    value: #utee_value_conv_codes,
                }
            });
        }

        const ORIGIN_PROPERTY_LEN: usize = 7;
        let property_len = ORIGIN_PROPERTY_LEN + conf.ext_properties.len();
        let no_mangle_attribute = self.edition.no_mangle_attribute_codes();
        self.code.extend(quote! {
        static FLAG_BOOL: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_SINGLE_INSTANCE) != 0;
        static FLAG_MULTI: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_MULTI_SESSION) != 0;
        static FLAG_INSTANCE: bool = (TA_FLAGS & optee_utee_sys::TA_FLAG_INSTANCE_KEEP_ALIVE) != 0;
        #no_mangle_attribute
        pub static ta_num_props: usize = #property_len;
        #no_mangle_attribute
        pub static ta_props: [optee_utee_sys::user_ta_property; #property_len] = [
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_SINGLE_INSTANCE,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
                value: &FLAG_BOOL as *const bool as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_MULTI_SESSION,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
                value: &FLAG_MULTI as *const bool as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_KEEP_ALIVE,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL,
                value: &FLAG_INSTANCE as *const bool as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_DATA_SIZE,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
                value: &TA_DATA_SIZE as *const u32 as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_STACK_SIZE,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32,
                value: &TA_STACK_SIZE as *const u32 as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_VERSION,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
                value: TA_VERSION as *const [u8] as *mut _,
            },
            optee_utee_sys::user_ta_property {
                name: optee_utee_sys::TA_PROP_STR_DESCRIPTION,
                prop_type: optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING,
                value: TA_DESCRIPTION as *const [u8] as *mut _,
            },
            #(#ext_property_codes),*
        ];
                });

        Ok(())
    }
    fn write_ta_head(&mut self, conf: &TaConfig) -> Result<(), Error> {
        let uuid_value_codes = uuid_to_tee_uuid_value_codes(&conf.uuid)?;
        let stack_size = conf.ta_stack_size + conf.ta_framework_stack_size;
        let no_mangle_attribute = self.edition.no_mangle_attribute_codes();
        let ta_head_session_attribute = self.edition.link_section_attribute_codes(".ta_head");
        self.code.extend(quote! {
        #no_mangle_attribute
        #ta_head_session_attribute
        pub static ta_head: optee_utee_sys::ta_head = optee_utee_sys::ta_head {
            uuid: #uuid_value_codes,
            stack_size: #stack_size,
            flags: TA_FLAGS,
            depr_entry: u64::MAX,
        };
                });

        Ok(())
    }
    fn write_ta_heap(&mut self) {
        let no_mangle_attribute = self.edition.no_mangle_attribute_codes();
        let bss_session_attribute = self.edition.link_section_attribute_codes(".bss");
        self.code.extend(quote! {
        #no_mangle_attribute
        #bss_session_attribute
        pub static ta_heap: [u8; TA_DATA_SIZE as usize] = [0; TA_DATA_SIZE as usize];

        #no_mangle_attribute
        pub static ta_heap_size: usize = mem::size_of::<u8>() * TA_DATA_SIZE as usize;
                })
    }
}

fn property_value_data_codes(value: &PropertyValue) -> Result<proc_macro2::TokenStream, Error> {
    match value {
        PropertyValue::U32(v) => Ok(quote! { #v }),
        PropertyValue::U64(v) => Ok(quote! { #v }),
        PropertyValue::Bool(v) => Ok(quote! { #v }),
        PropertyValue::Uuid(v) => uuid_to_tee_uuid_value_codes(v),
        PropertyValue::Str(v) => Ok(string_to_binary_codes(v)),
        PropertyValue::BinaryBlock(v) => Ok(string_to_binary_codes(v)),
        PropertyValue::Identity(login, uuid) => {
            identity_to_tee_identity_value_codes(*login, uuid)
        }
    }
}

fn uuid_to_tee_uuid_value_codes(uuid: &uuid::Uuid) -> Result<proc_macro2::TokenStream, Error> {
    let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = uuid.as_fields();
    Ok(quote! {
    optee_utee_sys::TEE_UUID {
        timeLow: #time_low,
        timeMid: #time_mid,
        timeHiAndVersion: #time_hi_and_version,
        clockSeqAndNode: [#(#clock_seq_and_node),* ],
    }
        })
}

fn identity_to_tee_identity_value_codes(
    login: u32,
    uuid: &uuid::Uuid,
) -> Result<proc_macro2::TokenStream, Error> {
    let tee_uuid_codes = uuid_to_tee_uuid_value_codes(uuid)?;
    Ok(quote! {
        optee_utee_sys::TEE_Identity {
            login: #login,
            uuid: #tee_uuid_codes,
        }
    })
}

fn property_value_rust_type_declaration_codes(value: &PropertyValue) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(match value {
        PropertyValue::U32(_) => "u32",
        PropertyValue::U64(_) => "u64",
        PropertyValue::Bool(_) => "bool",
        PropertyValue::Uuid(_) => "optee_utee_sys::TEE_UUID",
        PropertyValue::Str(_) => "&[u8]",
        PropertyValue::BinaryBlock(_) => "&[u8]",
        PropertyValue::Identity(..) => "optee_utee_sys::TEE_Identity",
    })
    .unwrap()
}

fn property_value_as_utee_value_codes(
    var_name: &str,
    value: &PropertyValue,
) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(
        match value {
            PropertyValue::U32(_) => format!("&{} as *const u32 as *mut _", var_name),
            PropertyValue::U64(_) => format!("&{} as *const u64 as *mut _", var_name),
            PropertyValue::Bool(_) => format!("&{} as *const bool as *mut _", var_name),
            PropertyValue::Uuid(_) => {
                format!("&{} as *const optee_utee_sys::TEE_UUID as *mut _", var_name)
            }
            PropertyValue::Str(_) => format!("{} as *const [u8] as *mut _", var_name),
            PropertyValue::BinaryBlock(_) => format!("{} as *const [u8] as *mut _", var_name),
            PropertyValue::Identity(..) => format!(
                "&{} as *const optee_utee_sys::TEE_Identity as *mut _",
                var_name
            ),
        }
        .as_str(),
    )
    .unwrap()
}

fn property_value_utee_type_codes(value: &PropertyValue) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(match value {
        PropertyValue::U32(_) => "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U32",
        PropertyValue::U64(_) => "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_U64",
        PropertyValue::Bool(_) => "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BOOL",
        PropertyValue::Uuid(_) => "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_UUID",
        PropertyValue::Str(_) => "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_STRING",
        PropertyValue::BinaryBlock(_) => {
            "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_BINARY_BLOCK"
        }
        PropertyValue::Identity(..) => {
            "optee_utee_sys::user_ta_prop_type::USER_TA_PROP_TYPE_IDENTITY"
        }
    })
    .unwrap()
}

fn string_to_binary_codes(s: &str) -> proc_macro2::TokenStream {
    let wrapped = format!("b\"{}\\0\"", s);
    proc_macro2::TokenStream::from_str(&wrapped).unwrap()
}

impl RustEdition {
    fn no_mangle_attribute_codes(&self) -> proc_macro2::TokenStream {
        match self {
            RustEdition::Before2024 => quote! { #[no_mangle] },
            RustEdition::Edition2024 => quote! { #[unsafe(no_mangle)] },
        }
    }
    fn link_section_attribute_codes(&self, session_name: &str) -> proc_macro2::TokenStream {
        match self {
            RustEdition::Before2024 => quote! { #[link_section = #session_name] },
            RustEdition::Edition2024 => quote! { #[unsafe(link_section = #session_name)] },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edition_before_2024() {
        let uuid = "26509cec-4a2b-4935-87ab-762d89fbf0b0";
        let conf = TaConfig::new_default(uuid, "0.1.0", "test_before_2024")
            .unwrap()
            .ta_data_size(1024 * 1024);
        let generator = HeaderFileGenerator::new(RustEdition::Before2024);
        let codes = generator.generate(&conf).unwrap();
        let exp_result = include_str!("../test_files/test_edition_before_2024_result.rs");
        assert_eq!(codes, exp_result);
    }
    #[test]
    fn test_edition_2024() {
        let uuid = "26509cec-4a2b-4935-87ab-762d89fbf0b0";
        let conf = TaConfig::new_default(uuid, "0.1.0", "test_edition_2024").unwrap();
        let generator = HeaderFileGenerator::new(RustEdition::Edition2024);
        let codes = generator.generate(&conf).unwrap();
        let exp_result = include_str!("../test_files/test_edition_2024_result.rs");
        assert_eq!(codes, exp_result);
    }
}
