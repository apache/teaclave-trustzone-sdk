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

use super::utee_types::utee_object_info;
use super::*;
use crate::size_t;
use core::ffi::*;

unsafe extern "C" {
    pub fn _utee_return(ret: c_ulong) -> !;
    pub fn _utee_log(buf: *const c_void, len: size_t);
    pub fn _utee_panic(code: c_ulong);
    pub fn _utee_get_property(
        prop_set: c_ulong,
        index: c_ulong,
        name: *mut c_void,
        name_len: *mut u32,
        buf: *mut c_void,
        blen: *mut u32,
        prop_type: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_get_property_name_to_index(
        prop_set: c_ulong,
        name: *const c_void,
        name_len: c_ulong,
        index: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_open_ta_session(
        dest: *const TEE_UUID,
        cancel_req_to: c_ulong,
        params: *mut utee_params,
        sess: *mut u32,
        ret_orig: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_close_ta_session(sess: c_ulong) -> TEE_Result;
    pub fn _utee_invoke_ta_command(
        sess: c_ulong,
        cancel_req_to: c_ulong,
        cmd_id: c_ulong,
        params: *mut utee_params,
        ret_orig: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_check_access_rights(flags: u32, buf: *const c_void, len: size_t) -> TEE_Result;
    pub fn _utee_get_cancellation_flag(cancel: *mut u32) -> TEE_Result;
    pub fn _utee_unmask_cancellation(old_mask: *mut u32) -> TEE_Result;
    pub fn _utee_mask_cancellation(old_mask: *mut u32) -> TEE_Result;
    pub fn _utee_wait(timeout: c_ulong) -> TEE_Result;
    pub fn _utee_get_time(cat: c_ulong, time: *mut TEE_Time) -> TEE_Result;
    pub fn _utee_set_ta_time(time: *const TEE_Time) -> TEE_Result;
    pub fn _utee_cryp_state_alloc(
        algo: c_ulong,
        op_mode: c_ulong,
        key1: c_ulong,
        key2: c_ulong,
        state: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_cryp_state_copy(dst: c_ulong, src: c_ulong) -> TEE_Result;
    pub fn _utee_cryp_state_free(state: c_ulong) -> TEE_Result;
    pub fn _utee_hash_init(state: c_ulong, iv: *const c_void, iv_len: size_t) -> TEE_Result;
    pub fn _utee_hash_update(
        state: c_ulong,
        chunk: *const c_void,
        chunk_size: size_t,
    ) -> TEE_Result;
    pub fn _utee_hash_final(
        state: c_ulong,
        chunk: *const c_void,
        chunk_size: size_t,
        hash: *mut c_void,
        hash_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_cipher_init(state: c_ulong, iv: *const c_void, iv_len: size_t) -> TEE_Result;
    pub fn _utee_cipher_update(
        state: c_ulong,
        src: *const c_void,
        src_len: size_t,
        dest: *mut c_void,
        dest_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_cipher_final(
        state: c_ulong,
        src: *const c_void,
        src_len: size_t,
        dest: *mut c_void,
        dest_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_cryp_obj_get_info(obj: c_ulong, info: *mut utee_object_info) -> TEE_Result;
    pub fn _utee_cryp_obj_restrict_usage(obj: c_ulong, usage: c_ulong) -> TEE_Result;
    pub fn _utee_cryp_obj_get_attr(
        obj: c_ulong,
        attr_id: c_ulong,
        buffer: *mut c_void,
        size: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_cryp_obj_alloc(ttype: c_ulong, max_size: c_ulong, obj: *mut u32) -> TEE_Result;
    pub fn _utee_cryp_obj_close(obj: c_ulong) -> TEE_Result;
    pub fn _utee_cryp_obj_reset(obj: c_ulong) -> TEE_Result;
    pub fn _utee_cryp_obj_populate(
        obj: c_ulong,
        attrs: *mut utee_attribute,
        attr_count: c_ulong,
    ) -> TEE_Result;
    pub fn _utee_cryp_obj_copy(dst_obj: c_ulong, src_obj: c_ulong) -> TEE_Result;
    pub fn _utee_cryp_obj_generate_key(
        obj: c_ulong,
        key_size: c_ulong,
        params: *const utee_attribute,
        param_count: c_ulong,
    ) -> TEE_Result;
    pub fn _utee_cryp_derive_key(
        state: c_ulong,
        params: *const utee_attribute,
        param_count: c_ulong,
        derived_key: c_ulong,
    ) -> TEE_Result;
    pub fn _utee_cryp_random_number_generate(buf: *mut c_void, blen: size_t) -> TEE_Result;
    pub fn _utee_authenc_init(
        state: c_ulong,
        nonce: *const c_void,
        nonce_len: size_t,
        tag_len: size_t,
        aad_len: size_t,
        payload_len: size_t,
    ) -> TEE_Result;
    pub fn _utee_authenc_update_aad(
        state: c_ulong,
        aad_data: *const c_void,
        aad_data_len: size_t,
    ) -> TEE_Result;
    pub fn _utee_authenc_update_payload(
        state: c_ulong,
        src_data: *const c_void,
        src_len: size_t,
        dest_data: *mut c_void,
        dest_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_authenc_enc_final(
        state: c_ulong,
        src_data: *const c_void,
        src_len: size_t,
        dest_data: *mut c_void,
        dest_len: *mut u64,
        tag: *mut c_void,
        tag_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_authenc_dec_final(
        state: c_ulong,
        src_data: *const c_void,
        src_len: size_t,
        dest_data: *mut c_void,
        dest_len: *mut u64,
        tag: *const c_void,
        tag_len: size_t,
    ) -> TEE_Result;
    pub fn _utee_asymm_operate(
        state: c_ulong,
        params: *const utee_attribute,
        num_params: c_ulong,
        src_data: *const c_void,
        src_len: size_t,
        dest_data: *mut c_void,
        dest_len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_asymm_verify(
        state: c_ulong,
        params: *const utee_attribute,
        num_params: c_ulong,
        data: *const c_void,
        data_len: size_t,
        sig: *const c_void,
        sig_len: size_t,
    ) -> TEE_Result;
    pub fn _utee_storage_obj_open(
        storage_id: c_ulong,
        object_id: *const c_void,
        object_id_len: size_t,
        flags: c_ulong,
        obj: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_storage_obj_create(
        storage_id: c_ulong,
        object_id: *const c_void,
        object_id_len: size_t,
        flags: c_ulong,
        attr: c_ulong,
        data: *const c_void,
        len: size_t,
        obj: *mut u32,
    ) -> TEE_Result;
    pub fn _utee_storage_obj_del(obj: c_ulong) -> TEE_Result;
    pub fn _utee_storage_obj_rename(
        obj: c_ulong,
        new_obj_id: *const c_void,
        new_obj_id_len: size_t,
    ) -> TEE_Result;
    pub fn _utee_storage_alloc_enum(obj_enum: *mut u32) -> TEE_Result;
    pub fn _utee_storage_free_enum(obj_enum: c_ulong) -> TEE_Result;
    pub fn _utee_storage_reset_enum(obj_enum: c_ulong) -> TEE_Result;
    pub fn _utee_storage_start_enum(obj_enum: c_ulong, storage_id: c_ulong) -> TEE_Result;
    pub fn _utee_storage_next_enum(
        obj_enum: c_ulong,
        info: *mut utee_object_info,
        obj_id: *mut c_void,
        len: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_storage_obj_read(
        obj: c_ulong,
        data: *mut c_void,
        len: size_t,
        count: *mut u64,
    ) -> TEE_Result;
    pub fn _utee_storage_obj_write(obj: c_ulong, data: *const c_void, len: size_t) -> TEE_Result;
    pub fn _utee_storage_obj_trunc(obj: c_ulong, len: size_t) -> TEE_Result;
    pub fn _utee_storage_obj_seek(obj: c_ulong, offset: i32, whence: c_ulong) -> TEE_Result;
    pub fn _utee_cache_operation(va: *mut c_void, l: size_t, op: c_ulong) -> TEE_Result;
    // unimplemented syscall
    // pub fn utee_gprof_send(buf: *mut c_void, size: size_t, id: *mut u32) -> TEE_Result;
}
