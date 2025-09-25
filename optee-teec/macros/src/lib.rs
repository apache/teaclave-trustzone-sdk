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
        /// ## Security assumptions
        /// The caller (OP-TEE OS) must ensure:
        /// - `data` points to valid memory for reads and writes of at least `in_len` bytes
        /// - `out_len` is a valid, writable, and properly aligned pointer to a `u32`
        /// - If `in_len == 0`, `data` may be null; otherwise it must be non-null
        ///
        /// Additional guarantees enforced by `PluginParameters` in this SDK:
        /// - The output length (`outslice.len()`) will never exceed `in_len`
        ///   because [`PluginParameters::set_buf_from_slice`] checks and rejects overflows.
        /// - Input and output share the same buffer (`inout`), so overlap is intentional
        ///   and safely handled by [`PluginParameters::set_buf_from_slice`].
        ///
        /// ## Scenarios
        /// - **Valid empty call**: `data = null`, `in_len = 0` → allowed; produces an empty output.
        /// - **Normal call**: `data` points to a buffer of size `in_len`; the plugin writes
        ///   up to `in_len` bytes and updates `*out_len`.
        /// - **Overflow attempt**: plugin inner logic (developer code) tries to return
        ///   more bytes than `in_len` → rejected by `set_buf_from_slice`, error returned.
        /// - **Invalid pointers**: if `data` or `out_len` are invalid (null, dangling, misaligned,
        ///   or pointing to read-only memory), dereferencing them causes undefined behavior.
        ///   This must be prevented by the caller (OP-TEE OS).
        pub unsafe fn _plugin_invoke(
            cmd: u32,
            sub_cmd: u32,
            data: *mut core::ffi::c_char,
            in_len: u32,
            out_len: *mut u32
        ) -> optee_teec::raw::TEEC_Result {
            fn inner(#params) -> optee_teec::Result<()> {
                #f_block
            }
            let mut inbuf = std::slice::from_raw_parts_mut(data, in_len as usize);
            let mut params = optee_teec::PluginParameters::new(cmd, sub_cmd, inbuf);
            if let Err(err) = inner(&mut params) {
                return err.raw_code();
            };
            let outslice = params.get_out_slice();
            *out_len = outslice.len() as u32;
            std::ptr::copy(outslice.as_ptr(), data, outslice.len());
            return optee_teec::raw::TEEC_SUCCESS;
        }
    )
    .into()
}
