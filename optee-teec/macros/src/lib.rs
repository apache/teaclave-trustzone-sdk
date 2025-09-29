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

#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;
use syn::spanned::Spanned;

/// Attribute to declare the init function of a plugin
/// ``` no_run
/// #[plugin_init]
/// fn plugin_init() -> Result<()> {}
/// ```
#[proc_macro_attribute]
pub fn plugin_init(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_vis = &f.vis;
    let f_block = &f.block;
    let f_sig = &f.sig;
    let f_inputs = &f_sig.inputs;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f_vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && f_inputs.is_empty()
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none()
        && check_return_type(&f);

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[plugin_init]` function must have signature `fn() -> optee_teec::Result<()>`",
        )
        .to_compile_error()
        .into();
    }

    quote!(
        pub fn _plugin_init() -> optee_teec::raw::TEEC_Result {
            fn inner() -> optee_teec::Result<()> {
                #f_block
            }
            match inner() {
                Ok(()) => optee_teec::raw::TEEC_SUCCESS,
                Err(err) => err.raw_code(),
            }
        }
    )
    .into()
}

// check if return_type of the function is `optee_teec::Result<()>`
fn check_return_type(item_fn: &syn::ItemFn) -> bool {
    if let syn::ReturnType::Type(_, return_type) = item_fn.sig.output.to_owned() {
        if let syn::Type::Path(path) = return_type.as_ref() {
            let expected_type = quote! { optee_teec::Result<()> };
            let actual_type = path.path.to_token_stream();
            if expected_type.to_string() == actual_type.to_string() {
                return true;
            }
        }
    }
    false
}

/// Attribute to declare the invoke function of a plugin
/// ``` no_run
/// #[plugin_invoke]
/// fn plugin_invoke(params: &mut PluginParameters) {}
/// ```
#[proc_macro_attribute]
pub fn plugin_invoke(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_vis = &f.vis;
    let f_block = &f.block;
    let f_sig = &f.sig;
    let f_inputs = &f_sig.inputs;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f_vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && f_inputs.len() == 1
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none()
        && check_return_type(&f);

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            concat!(
                "`#[plugin_invoke]` function must have signature",
                " `fn(params: &mut PluginParameters) -> optee_teec::Result<()>`"
            ),
        )
        .to_compile_error()
        .into();
    }

    let params = f_inputs
        .first()
        .expect("we have already verified its len")
        .into_token_stream();

    quote!(
        /// # Safety
        ///
        /// The `_plugin_invoke` function is the `extern "C"` entrypoint called by OP-TEE OS.
        /// This SDK allows developers to implement the inner logic for a Normal World plugin in Rust.
        /// More about plugins:
        /// https://optee.readthedocs.io/en/latest/architecture/globalplatform_api.html#loadable-plugins-framework
        ///
        /// According to Clippy checks, any FFI function taking raw pointers as parameters
        /// must be marked `unsafe`. This applies here because the function directly
        /// dereferences `data` and `out_len`.
        ///
        /// ## Security Assumptions
        /// The caller (OP-TEE OS) must ensure:
        /// - `data` points to valid memory for reads and writes of at least `in_len` bytes
        /// - `out_len` is a valid, writable, and properly aligned pointer to a `u32` (cannot be null)
        /// - If `in_len == 0`, `data` may be null; otherwise it must be non-null
        /// - The memory region pointed to by `data` must not be modified by other threads
        ///   or processes during plugin execution
        ///
        /// Additional guarantees enforced by `PluginParameters` in this SDK:
        /// - If `data` is null and `in_len` is 0, it is treated as an empty input buffer;
        ///   the inner logic (developer code) should consider this case
        /// - If the output length exceeds `in_len`, it will be rejected and a short buffer
        ///   error returned, with the required `out_len` set
        /// - Input and output share the same buffer, so overlap is intentional and safely
        ///   handled by [`PluginParameters::set_buf_from_slice`] when `out_len <= in_len`
        /// - If no output is set for a success call, `out_len` will be `0`
        ///
        /// ## Usage Scenarios
        /// - **Valid empty call**: `data = null`, `in_len = 0` → allowed (empty input to inner)
        /// - **Normal call**: `data` points to a buffer of size `in_len`; if `out_len <= in_len`,
        ///   the plugin writes up to `in_len` bytes and updates `*out_len`; if `out_len > in_len`,
        ///   it is rejected with a short buffer error
        /// - **Buffer overflow attempt**: if inner logic (developer code) tries to return
        ///   more bytes than `in_len` → rejected by `set_buf_from_slice`, error returned with required `out_len`
        /// - **Invalid pointers**: null pointers are checked, but other invalid cases of pointers
        ///   such as dangling, misaligned, or read-only pointers will cause undefined behavior 
        ///   and must be prevented by the caller
        pub unsafe fn _plugin_invoke(
            cmd: u32,
            sub_cmd: u32,
            data: *mut core::ffi::c_char,
            in_len: u32,
            out_len: *mut u32,
        ) -> optee_teec::raw::TEEC_Result {
            fn inner(#params) -> optee_teec::Result<()> {
                #f_block
            }

            // Check for null pointers
            if data.is_null() && in_len != 0 {
                return optee_teec::raw::TEEC_ERROR_BAD_PARAMETERS;
            }
            if out_len.is_null() {
                return optee_teec::raw::TEEC_ERROR_BAD_PARAMETERS;
            }

            // Prepare input buffer
            // `data` is guaranteed to be non-null if `in_len > 0` (checked above)
            // If `data` is null, `in_len` must be 0, so we create an empty slice
            // Otherwise, we create a mutable slice from the raw pointer and length
            let inbuf = if data.is_null() {
                &mut []
            } else {
                // SAFETY: from_raw_parts_mut() is unsafe, but avoids copying the memory
                // (which is unacceptable for large io buffers).
                // Note that the caller must ensure the memory is consistent during the plugin execution.
                std::slice::from_raw_parts_mut(data, in_len as usize)
            };

            let mut params = optee_teec::PluginParameters::new(cmd, sub_cmd, inbuf);
            match inner(&mut params) {
                Ok(()) => {
                    *out_len = params.get_required_out_len() as u32;
                    optee_teec::raw::TEEC_SUCCESS
                }
                Err(err) => {
                    if err.kind() == optee_teec::ErrorKind::ShortBuffer {
                        // Inform the caller about the required buffer size
                        *out_len = params.get_required_out_len() as u32;
                        optee_teec::raw::TEEC_ERROR_SHORT_BUFFER
                    } else {
                        err.raw_code()
                    }
                }
            }
        }
    )
    .into()
}
