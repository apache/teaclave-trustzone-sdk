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
    let mut cfg = ctest::TestGenerator::new();
    let ta_include_path = {
        let optee_os_dir = env::var("TA_DEV_KIT_DIR").expect("TA_DEV_KIT_DIR not set");
        let include_path = PathBuf::from(optee_os_dir).join("include");
        if !include_path.exists() {
            panic!(
                "TA DevKit include path {} does not exist",
                include_path.display()
            );
        }
        include_path
    };
    cfg.header("tee_api_types.h")
        .header("tee_api_defines.h")
        .header("utee_types.h")
        .header("user_ta_header.h")
        .header("tee_api.h")
        .header("utee_syscalls.h")
        .header("tee_tcpsocket.h")
        .header("tee_udpsocket.h")
        .header("tee_internal_api_extensions.h")
        .header("__tee_tcpsocket_defines_extensions.h")
        .include(ta_include_path.display().to_string())
        .type_name(|s, _is_struct, _is_union| {
            if s == "utee_params"
                || s == "ta_head"
                || s == "utee_attribute"
                || s == "utee_object_info"
                || s == "user_ta_property"
                || s == "TEE_tcpSocket_Setup_s"
                || s == "TEE_udpSocket_Setup_s"
            {
                return format!("struct {}", s);
            }
            s.to_string()
        });
    cfg.skip_struct(|s| {
        s == "Memref"
            || s == "Value"
            || s == "content"
            || s.ends_with("Handle")
            || s == "ta_prop"
            || s == "user_ta_property"
            || s == "TEE_iSocket_s" // untestable due to `const struct`
    });
    cfg.skip_field(|s, field| {
        (s == "ta_head" && field == "entry")
            || field == "content"
            || field == "value"
            || field == "memref"
            || field == "keyInformation"
    });
    cfg.skip_type(|s| s == "Memref" || s == "Value");
    cfg.skip_fn(|s| s == "TEE_BigIntFMMConvertToBigInt" || s == "__utee_entry");
    cfg.skip_const(|s| s.starts_with("TA_PROP_STR") || s == "TEE_HANDLE_NULL");
    cfg.skip_roundtrip(|s| s.starts_with("TEE_") || s.starts_with("utee_") || s == "ta_head");
    cfg.skip_static(|s| s == "TEE_tcpSocket" || s == "TEE_udpSocket");
    cfg.generate("../optee-utee-sys/src/lib.rs", "all.rs");
    println!("cargo:rustc-link-lib=static=mbedtls");
    println!("cargo:rustc-link-lib=static=utee");
    println!("cargo:rustc-link-lib=static=utils");
    println!("cargo:rustc-link-lib=static=dl");

    let out_dir = env::var("OUT_DIR").unwrap();
    let undefined_path = PathBuf::from(&out_dir).join("undefined.c");

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

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=undefined");
}
