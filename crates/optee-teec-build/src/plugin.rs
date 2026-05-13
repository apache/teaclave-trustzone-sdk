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

use std::path::PathBuf;
pub use uuid;

/// Default name for the plugin init function.
pub const DEFAULT_INIT_FN_NAME: &str = "__plugin_bindgen_init";
/// Default name for the plugin invoke function.
pub const DEFAULT_INVOKE_FN_NAME: &str = "__plugin_bindgen_invoke";

/// Configuration for building an OP-TEE supplicant plugin binding.
///
/// Holds the plugin name, UUID, init/invoke function names, and optional
/// output destination. Use the builder-style API (`with_*` methods) to
/// customize defaults, then call [`PluginConfig::build`] to generate the
/// `plugin_static.rs` file.
pub struct PluginConfig {
    name: String,
    uuid: uuid::Uuid,
    init_fn_name: String,
    invoke_fn_name: String,
    dest: Option<PathBuf>,
}

impl PluginConfig {
    /// Creates a new `PluginConfig` with the given UUID.
    ///
    /// The plugin name defaults to `CARGO_PKG_NAME`, and the init/invoke
    /// function names default to [`DEFAULT_INIT_FN_NAME`] and
    /// [`DEFAULT_INVOKE_FN_NAME`] respectively.
    pub fn new(uuid: uuid::Uuid) -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            uuid,
            init_fn_name: DEFAULT_INIT_FN_NAME.to_owned(),
            invoke_fn_name: DEFAULT_INVOKE_FN_NAME.to_owned(),
            dest: None,
        }
    }
    /// Sets a custom plugin name.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    /// Sets a custom init function name (overriding [`DEFAULT_INIT_FN_NAME`]).
    pub fn with_init_fn_name(mut self, fn_name: &str) -> Self {
        self.init_fn_name = fn_name.to_string();
        self
    }
    /// Sets a custom invoke function name (overriding [`DEFAULT_INVOKE_FN_NAME`]).
    pub fn with_invoke_fn_name(mut self, fn_name: &str) -> Self {
        self.invoke_fn_name = fn_name.to_string();
        self
    }
    /// Sets a custom output file path
    pub fn with_dest(mut self, out_path: PathBuf) -> Self {
        self.dest = Some(out_path);
        self
    }
    /// Generates the plugin binding source and writes it to the output file.
    ///
    /// If the output file already exists with identical content, the write is
    /// skipped. The default output path is `$OUT_DIR/plugin_static.rs`.
    pub fn build(&self) -> std::io::Result<()> {
        let codes = generate_binding(
            &self.name,
            &self.uuid,
            &self.init_fn_name,
            &self.invoke_fn_name,
        )
        .to_string();
        let out_path = self.get_out_path();
        if let Ok(v) = std::fs::read(&out_path)
            && v.eq(codes.as_bytes())
        {
            return Ok(());
        }

        if let Some(parent_dir) = out_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }
        std::fs::write(out_path, codes.as_bytes())
    }
}

impl PluginConfig {
    /// Returns the output file path for the generated binding.
    ///
    /// Uses the custom destination if set, otherwise defaults to
    /// `$OUT_DIR/plugin_static.rs`.
    fn get_out_path(&self) -> PathBuf {
        match self.dest.as_ref() {
            Some(v) => v.clone(),
            None => {
                let out_dir = PathBuf::from(
                    std::env::var("OUT_DIR").expect("Infallible when using in build.rs"),
                );
                out_dir.join("plugin_static.rs")
            }
        }
    }
}

fn generate_binding(
    name: &str,
    uuid: &uuid::Uuid,
    init_fn_name: &str,
    invoke_fn_name: &str,
) -> proc_macro2::TokenStream {
    let (uuid_f1, uuid_f2, uuid_f3, uuid_f4) = uuid.as_fields();
    let name_bytes_with_null = format!("{}\0", name);
    let init_fn_name = quote::format_ident!("{}", init_fn_name);
    let invoke_fn_name = quote::format_ident!("{}", invoke_fn_name);
    quote::quote! {
        const _: () = {
            use optee_teec::raw::{PluginMethod, TEEC_UUID};

            static PLUGIN_NAME: &str = #name_bytes_with_null;

            #[unsafe(no_mangle)]
            pub static mut plugin_method: PluginMethod = PluginMethod {
                name: PLUGIN_NAME.as_ptr() as *const _,
                uuid: TEEC_UUID {
                    timeLow: #uuid_f1,
                    timeMid: #uuid_f2,
                    timeHiAndVersion: #uuid_f3,
                    clockSeqAndNode: [#(#uuid_f4),*],
                },
                init: #init_fn_name,
                invoke: #invoke_fn_name,
            };
        };
    }
}
