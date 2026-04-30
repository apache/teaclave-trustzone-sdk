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

extern crate proc_macro;

use optee_teec_build::{DEFAULT_INIT_FN_NAME, DEFAULT_INVOKE_FN_NAME};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, parse_macro_input};

/// Attribute to derive the injected init function from an existing function
/// ``` no_run
/// use optee_teec_macros::plugin_init;
///
/// #[plugin_init]
/// fn plugin_init() -> optee_teec::Result<()> {
///     Ok(())
/// }
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

    let bindgen_fn_name = quote::format_ident!("{}", DEFAULT_INIT_FN_NAME);
    let origin_fn_name = &f_sig.ident;
    quote!(
        #f_vis #f_sig {
            #f_block
        }
        const _: fn() -> optee_teec::Result<()> = #origin_fn_name;
        unsafe extern "C" fn #bindgen_fn_name() -> optee_teec::raw::TEEC_Result {
            match #origin_fn_name() {
                Ok(()) => optee_teec::raw::TEEC_SUCCESS,
                Err(err) => err.raw_code(),
            }
        }
    )
    .into()
}

// check if return_type of the function is `optee_teec::Result<()>`
fn check_return_type(item_fn: &syn::ItemFn) -> bool {
    const EXPECTED: [&str; 2] = ["optee_teec", "Result"];
    if let syn::ReturnType::Type(_, return_type) = item_fn.sig.output.to_owned()
        && let syn::Type::Path(path) = return_type.as_ref()
    {
        return check_path_might_match(&path.path, &EXPECTED);
    }
    false
}

// check path match the expected values
// it might still fail if developers re-export crate as other name.
fn check_path_might_match(path: &syn::Path, exp: &[&str]) -> bool {
    path.segments.len() <= exp.len()
        && path
            .segments
            .iter()
            .zip(&exp[exp.len() - path.segments.len()..])
            .all(|(seg, exp)| seg.ident == *exp)
}

fn check_invoke_fn_params(item_fn: &syn::ItemFn) -> bool {
    if item_fn.sig.inputs.len() != 1 {
        return false;
    }

    let arg = item_fn.sig.inputs.first().expect("Infallible");
    if let FnArg::Typed(typ) = arg
        && let syn::Type::Reference(typ_ref) = typ.ty.as_ref()
        && typ_ref.mutability.is_some()
        && let syn::Type::Path(inner_type) = typ_ref.elem.as_ref()
    {
        const EXPECTED: [&str; 2] = ["optee_teec", "PluginParameters"];
        return check_path_might_match(&inner_type.path, &EXPECTED);
    }
    false
}

/// Attribute to derive the injected invoke function from an existing function
/// ``` no_run
/// use optee_teec_macros::plugin_invoke;
///
/// #[plugin_invoke]
/// fn plugin_invoke(params: &mut optee_teec::PluginParameters) -> optee_teec::Result<()> {
///     Ok(())
/// }
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
        && check_return_type(&f)
        && check_invoke_fn_params(&f);

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

    let bindgen_fn_name = quote::format_ident!("{}", DEFAULT_INVOKE_FN_NAME);
    let origin_fn_name = &f_sig.ident;

    quote!(
        #f_vis #f_sig {
            #f_block
        }
        const _: fn(_: &mut optee_teec::PluginParameters) -> optee_teec::Result<()> = #origin_fn_name;

        unsafe extern "C" fn #bindgen_fn_name(
            cmd: u32,
            sub_cmd: u32,
            data: *mut core::ffi::c_void,
            in_len: optee_teec::raw::size_t,
            out_len: *mut optee_teec::raw::size_t,
        ) -> optee_teec::raw::TEEC_Result {
            let mut parameter = match unsafe {
                optee_teec::PluginParameters::from_raw(
                    cmd,
                    sub_cmd,
                    data,
                    in_len,
                    out_len,
                )
            } {
                Ok(v) => v,
                Err(err) => return err.raw_code(),
            };

            match #origin_fn_name(&mut parameter) {
                Ok(()) => optee_teec::raw::TEEC_SUCCESS,
                Err(err) => err.raw_code(),
            }
        }
    )
    .into()
}
