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

extern crate alloc;
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

/// Attribute to declare the entry point of creating TA.
///
/// # Examples
///
/// ``` no_run
/// #[ta_create]
/// fn ta_create() -> Result<()> { }
/// ```
#[proc_macro_attribute]
pub fn ta_create(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_sig = &f.sig;
    let f_ident = &f_sig.ident;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && f_sig.inputs.is_empty()
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none();

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[ta_crate]` function must have signature `fn() -> optee_utee::Result<()>`",
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[no_mangle]
        pub extern "C" fn TA_CreateEntryPoint() -> optee_utee_sys::TEE_Result {
            match #f_ident() {
                Ok(_) => optee_utee_sys::TEE_SUCCESS,
                Err(e) => e.raw_code()
            }
        }

        #f
    )
    .into()
}

/// Attribute to declare the entry point of destroying TA.
///
/// # Examples
///
/// ``` no_run
/// #[ta_destroy]
/// fn ta_destroy() { }
/// ```
#[proc_macro_attribute]
pub fn ta_destroy(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_sig = &f.sig;
    let f_ident = &f_sig.ident;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && f_sig.inputs.is_empty()
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none()
        && matches!(f_sig.output, syn::ReturnType::Default);

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[ta_destroy]` function must have signature `fn()`",
        )
        .to_compile_error()
        .into();
    }

    quote!(
        #[no_mangle]
        pub extern "C" fn TA_DestroyEntryPoint() {
            #f_ident()
        }

        #f
    )
    .into()
}

/// Attribute to declare the entry point of opening a session. Pointer to
/// session context pointer (*mut *mut T) can be defined as an optional
/// parameter.
///
/// # Examples
///
/// ``` no_run
/// #[ta_open_session]
/// fn open_session(params: &mut Parameters) -> Result<()> { }
///
/// // T is the sess_ctx struct and is required to implement default trait
/// #[ta_open_session]
/// fn open_session(params: &mut Parameters, sess_ctx: &mut T) -> Result<()> { }
/// ```
#[proc_macro_attribute]
pub fn ta_open_session(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_sig = &f.sig;
    let f_ident = &f_sig.ident;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && (f_sig.inputs.len() == 1 || f_sig.inputs.len() == 2)
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none();

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[ta_open_session]` function must have signature `fn(&mut Parameters) -> Result<()>` or `fn(&mut Parameters, &mut T) -> Result<()>`",
        )
        .to_compile_error()
        .into();
    }

    match f_sig.inputs.len() {
        1 => quote!(
            #[no_mangle]
            pub extern "C" fn TA_OpenSessionEntryPoint(
                param_types: optee_utee::RawParamTypes,
                params: &mut optee_utee::RawParams,
                _: *mut *mut core::ffi::c_void,
            ) -> optee_utee_sys::TEE_Result {
                let mut parameters = Parameters::from_raw(params, param_types);
                match #f_ident(&mut parameters) {
                    Ok(_) => optee_utee_sys::TEE_SUCCESS,
                    Err(e) => e.raw_code()
                }
            }

            #f
        )
        .into(),

        2 => {
            let ctx_type = match extract_fn_arg_mut_ref_type(&f_sig.inputs[1]) {
                Ok(v) => v,
                Err(e) => return e.to_compile_error().into(),
            };

            quote!(
                // To eliminate the clippy error: this public function might dereference a raw pointer but is not marked `unsafe`
                // we just expand the unsafe block, but the session-related macros need refactoring in the future
                #[no_mangle]
                pub unsafe extern "C" fn TA_OpenSessionEntryPoint(
                    param_types: optee_utee::RawParamTypes,
                    params: &mut optee_utee::RawParams,
                    sess_ctx: *mut *mut core::ffi::c_void,
                ) -> optee_utee_sys::TEE_Result {
                    let mut parameters = Parameters::from_raw(params, param_types);
                    let mut ctx: #ctx_type = Default::default();
                    match #f_ident(&mut parameters, &mut ctx) {
                        Ok(_) =>
                        {
                            *sess_ctx = Box::into_raw(Box::new(ctx)) as _;
                            optee_utee_sys::TEE_SUCCESS
                        }
                        Err(e) => e.raw_code()
                    }
                }

                #f
            )
            .into()
        }
        _ => unreachable!(),
    }
}

/// Attribute to declare the entry point of closing a session. Session context
/// raw pointer (`*mut T`) can be defined as an optional parameter.
///
/// # Examples
///
/// ``` no_run
/// #[ta_close_session]
/// fn close_session(sess_ctx: &mut T) { }
///
/// #[ta_close_session]
/// fn close_session() { }
/// ```
#[proc_macro_attribute]
pub fn ta_close_session(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_sig = &f.sig;
    let f_ident = &f_sig.ident;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && (f_sig.inputs.is_empty() || f_sig.inputs.len() == 1)
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none()
        && matches!(f_sig.output, syn::ReturnType::Default);

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[ta_close_session]` function must have signature `fn(&mut T)` or `fn()`",
        )
        .to_compile_error()
        .into();
    }

    match f_sig.inputs.len() {
        0 => quote!(
            #[no_mangle]
            pub extern "C" fn TA_CloseSessionEntryPoint(_: *mut core::ffi::c_void) {
                #f_ident()
            }

            #f
        )
        .into(),
        1 => {
            let ctx_type = match extract_fn_arg_mut_ref_type(&f_sig.inputs[0]) {
                Ok(v) => v,
                Err(e) => return e.to_compile_error().into(),
            };

            quote!(
                // To eliminate the clippy error: this public function might dereference a raw pointer but is not marked `unsafe`
                // we just expand the unsafe block, but the session-related macros need refactoring in the future
                #[no_mangle]
                pub unsafe extern "C" fn TA_CloseSessionEntryPoint(sess_ctx: *mut core::ffi::c_void) {
                    if sess_ctx.is_null() {
                        panic!("sess_ctx is null");
                    }
                    let mut b = Box::from_raw(sess_ctx as *mut #ctx_type);
                    #f_ident(&mut b);
                    drop(b);
                }

                #f
            )
            .into()
        }
        _ => unreachable!(),
    }
}

/// Attribute to declare the entry point of invoking commands. Session context
/// reference (`&mut T`) can be defined as an optional parameter.
///
/// # Examples
///
/// ``` no_run
/// #[ta_invoke_command]
/// fn invoke_command(sess_ctx: &mut T, cmd_id: u32, params: &mut Parameters) -> Result<()> { }
///
/// #[ta_invoke_command]
/// fn invoke_command(cmd_id: u32, params: &mut Parameters) -> Result<()> { }
/// ```
#[proc_macro_attribute]
pub fn ta_invoke_command(_args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as syn::ItemFn);
    let f_sig = &f.sig;
    let f_ident = &f_sig.ident;

    // check the function signature
    let valid_signature = f_sig.constness.is_none()
        && matches!(f.vis, syn::Visibility::Inherited)
        && f_sig.abi.is_none()
        && (f_sig.inputs.len() == 2 || f_sig.inputs.len() == 3)
        && f_sig.generics.where_clause.is_none()
        && f_sig.variadic.is_none();

    if !valid_signature {
        return syn::parse::Error::new(
            f.span(),
            "`#[ta_invoke_command]` function must have signature `fn(&mut T, u32, &mut Parameters) -> Result<()>` or `fn(u32, &mut Parameters) -> Result<()>`",
        )
        .to_compile_error()
        .into();
    }

    match f_sig.inputs.len() {
        2 => quote!(
            #[no_mangle]
            pub extern "C" fn TA_InvokeCommandEntryPoint(
                _: *mut core::ffi::c_void,
                cmd_id: u32,
                param_types: u32,
                params: &mut optee_utee::RawParams,
            ) -> optee_utee_sys::TEE_Result {
                let mut parameters = Parameters::from_raw(params, param_types);
                match #f_ident(cmd_id, &mut parameters) {
                    Ok(_) => {
                        optee_utee_sys::TEE_SUCCESS
                    },
                    Err(e) => e.raw_code()
                }
            }

            #f
        )
        .into(),
        3 => {
            let ctx_type = match extract_fn_arg_mut_ref_type(&f_sig.inputs[0]) {
                Ok(v) => v,
                Err(e) => return e.to_compile_error().into(),
            };

            quote!(
                // To eliminate the clippy error: this public function might dereference a raw pointer but is not marked `unsafe`
                // we just expand the unsafe block, but the session-related macros need refactoring in the future
                #[no_mangle]
                pub unsafe extern "C" fn TA_InvokeCommandEntryPoint(
                    sess_ctx: *mut core::ffi::c_void,
                    cmd_id: u32,
                    param_types: u32,
                    params: &mut optee_utee::RawParams,
                ) -> optee_utee_sys::TEE_Result {
                    if sess_ctx.is_null() {
                        return optee_utee_sys::TEE_ERROR_SECURITY;
                    }
                    let mut parameters = Parameters::from_raw(params, param_types);
                    let mut b = Box::from_raw(sess_ctx as *mut #ctx_type);
                    match #f_ident(&mut b, cmd_id, &mut parameters) {
                        Ok(_) => {
                            core::mem::forget(b);
                            optee_utee_sys::TEE_SUCCESS
                        },
                        Err(e) => {
                            core::mem::forget(b);
                            e.raw_code()
                        }
                    }
                }

                #f
            )
            .into()
        }
        _ => unreachable!(),
    }
}

fn extract_fn_arg_mut_ref_type(fn_arg: &syn::FnArg) -> Result<&syn::Type, syn::parse::Error> {
    if let syn::FnArg::Typed(ty) = fn_arg {
        if let syn::Type::Reference(type_ref) = ty.ty.as_ref() {
            if type_ref.mutability.is_some() {
                return Ok(&*type_ref.elem);
            }
        }
    };
    Err(syn::parse::Error::new(
        fn_arg.span(),
        "this argument should have signature `_: &mut T`",
    ))
}
