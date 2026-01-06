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

#[cfg_attr(feature = "mock", mockall::automock)]
pub mod api {
    use crate::*;
    use core::ffi::*;

    extern "C" {
        /// tee_invoke_supp_plugin() - invoke a tee-supplicant's plugin
        /// @uuid:       uuid of the plugin
        /// @cmd:        command for the plugin
        /// @sub_cmd:    subcommand for the plugin
        /// @buf:        data [for/from] the plugin [in/out]
        /// @len:        length of the input buf
        /// @outlen:     pointer to length of the output data (if they will be used)
        /// Return TEE_SUCCESS on success or TEE_ERRROR_* on failure.
        pub fn tee_invoke_supp_plugin(
            uuid: *const TEE_UUID,
            cmd: u32,
            sub_cmd: u32,
            buf: *mut c_void,
            len: usize,
            outlen: *mut usize,
        ) -> TEE_Result;

    }
}
