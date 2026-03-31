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

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let ta_include_path = {
        let ta_dev_kit_dir = env::var("TA_DEV_KIT_DIR").expect("TA_DEV_KIT_DIR not set");
        let include_path = PathBuf::from(ta_dev_kit_dir).join("include");
        if !include_path.exists() {
            panic!(
                "TA_DEV_KIT_DIR include path {} does not exist",
                include_path.display()
            );
        }
        include_path
    };
    let mut cfg = generate_cfg(ta_include_path.clone());
    ctest::generate_test(&mut cfg, "../optee-utee-sys/src/lib.rs", "all.rs").unwrap();
    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=utee");
    println!("cargo:rustc-link-lib=static=utils");
    println!("cargo:rustc-link-lib=static=dl");

    build_and_link_undefined(ta_include_path);
}

fn generate_cfg(ta_include_path: PathBuf) -> ctest::TestGenerator {
    let mut cfg = ctest::TestGenerator::new();
    cfg.edition(2024)
        .language(ctest::Language::C)
        .header("tee_api_types.h")
        .header("tee_api_defines.h")
        .header("utee_types.h")
        .header("user_ta_header.h")
        .header("tee_api.h")
        .header("utee_syscalls.h")
        .header("tee_tcpsocket.h")
        .header("tee_udpsocket.h")
        .header("tee_internal_api.h")
        .header("tee_internal_api_extensions.h")
        .header("__tee_tcpsocket_defines_extensions.h")
        .include(ta_include_path.display().to_string())
        .rename_type(|s| match s {
            "u64" => Some("uint64_t".to_string()),
            "u32" => Some("uint32_t".to_string()),
            "u16" => Some("uint16_t".to_string()),
            "c_char" => Some("char".to_string()),
            _ => None,
        })
        .rename_struct_ty(|ty| {
            if ty.starts_with("TEE") {
                return Some(ty.to_string());
            }
            None
        })
        .skip_struct(|s| {
            let s = s.ident();
            match s {
                "Memref"
                | "Value"
                | "ta_prop"
                | "user_ta_property"
                | "TEE_iSocket_s"
                | "TEE_udpSocket_Setup_s"
                | "TEE_tcpSocket_Setup_s"
                | "TEE_SEReaderProperties"
                | "TEE_SEAID" => true,
                _ => s.ends_with("Handle"),
            }
        })
        .skip_struct_field(|s, field| {
            let s = s.ident();
            let field = field.ident();
            (s == "ta_head" && field == "entry") 
                || (s == "TEE_OperationInfoMultiple" && field == "keyInformation") // va_list
                || (s == "TEE_Attribute" && field == "content") // anonymous union fields
        })
        .skip_fn(|s| {
            let s = s.ident();
            match s {
                "__utee_entry" => true,
                _ => false,
            }
        })
        .skip_union(|s| match s.ident() {
            "content" => true,
            _ => false,
        })
        .skip_union_field(|s, _field| {
            let s = s.ident();
            s == "TEE_Param" // TEE_Param only have anonymous fields
        })
        .rename_union_ty(|s| match s {
            "TEE_Param" => Some(s.to_string()),
            _ => None,
        });
    cfg
}

fn build_and_link_undefined(ta_include_path: PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let undefined_path = out_dir.join("undefined.c");

    let mut buffer = File::create(&undefined_path).unwrap();
    write!(
        buffer,
        "
        #include <tee_api_types.h>
        void* ta_props = 0;
        void* ta_num_props = 0;
        void* trace_level = 0;
        void* trace_ext_prefix = 0;
        void* ta_head = 0;
        void* ta_heap = 0;
        size_t ta_heap_size = 0;
        void TA_DestroyEntryPoint(void) {{}};
        TEE_Result tee_uuid_from_str(TEE_UUID __unused *uuid, const char __unused *s) {{
            return TEE_SUCCESS;
        }};
        int tahead_get_trace_level(void) {{
            return 0;
        }};
        TEE_Result TA_OpenSessionEntryPoint(uint32_t __unused pt,
				    TEE_Param __unused params[TEE_NUM_PARAMS],
				    void __unused **sess_ctx) {{
            return TEE_SUCCESS;
        }};
        void TA_CloseSessionEntryPoint(void *sess __unused) {{}};
        TEE_Result TA_CreateEntryPoint(void) {{
	        return TEE_SUCCESS;
        }}
        TEE_Result TA_InvokeCommandEntryPoint(void __unused *sess_ctx,
                    uint32_t __unused cmd_id,
				    uint32_t __unused pt,
				    TEE_Param __unused params[TEE_NUM_PARAMS]) {{
            return TEE_SUCCESS;
        }};
     "
    )
    .unwrap();

    let mut builder = cc::Build::new();
    builder
        .include(ta_include_path.display().to_string())
        .file(&undefined_path.display().to_string())
        .compile("undefined");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=undefined");
}
