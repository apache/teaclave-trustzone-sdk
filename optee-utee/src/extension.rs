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

use crate::{Error, ErrorKind, Result, Uuid};
use optee_utee_sys as raw;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::borrow::ToOwned;

pub struct LoadablePlugin {
    uuid: Uuid
}

pub struct LoadablePluginCommand<'a> {
    plugin: &'a LoadablePlugin,
    cmd_id: u32,
    sub_cmd_id: u32,
    buffer: Vec<u8>,
}

impl LoadablePlugin {
    pub fn new(uuid: &Uuid) -> Self {
        Self { uuid: uuid.to_owned() }
    }
    /// Invoke plugin with given request data, use when you want to post something into REE.
    /// ``` rust,no_run
    /// # use optee_utee::{LoadablePlugin, Uuid};
    /// # fn main() -> optee_utee::Result<()> {
    /// # let uuid = Uuid::parse_str("").unwrap();
    /// # let command_id = 0;
    /// # let subcommand_id = 0;
    /// # let request_data = [0_u8; 0];
    /// let plugin = LoadablePlugin::new(&uuid);
    /// let result = plugin.invoke(command_id, subcommand_id, &request_data)?;
    /// # Ok(())
    /// # }
    /// ```
    /// Caution: the size of the shared buffer is set to the len of data, you could get a 
    ///          ShortBuffer error if Plugin return more data than shared buffer, in that case,
    ///          use invoke_with_capacity and set the capacity manually.
    pub fn invoke(&self, command_id: u32, subcommand_id: u32, data: &[u8]) -> Result<Vec<u8>> {
        self.invoke_with_capacity(command_id, subcommand_id, data.len())
            .chain_write_body(data)
            .call()
    }
    /// Construct a command with shared buffer up to capacity size, write the buffer and call it
    /// manually, use when you need to control details of the invoking process.
    /// ```no_run
    /// # use optee_utee::{Uuid, LoadablePlugin};
    /// # fn main() -> optee_utee::Result<()> {
    /// # let plugin = LoadablePlugin::new(&Uuid::parse_str("").unwrap());
    /// # let request_data = [0_u8; 0];
    /// # let command_id = 0;
    /// # let sub_command_id = 0;
    /// # let capacity = 0;
    /// let mut cmd = plugin.invoke_with_capacity(command_id, sub_command_id, capacity);
    /// cmd.write_body(&request_data);
    /// let result = cmd.call()?;
    /// # Ok(())
    /// # }
    /// ```
    /// You can also imply a wrapper for performance, for example, imply a std::io::Write so
    /// serde_json can write to the buffer directly.
    /// ```no_run
    /// # use optee_utee::{LoadablePluginCommand, Uuid, LoadablePlugin, trace_println};
    /// # use optee_utee::ErrorKind;
    /// # fn main() -> optee_utee::Result<()> { 
    /// # let command_id = 0;
    /// # let subcommand_id = 0;
    /// # let capacity = 0;
    /// # let plugin = LoadablePlugin::new(&Uuid::parse_str("").unwrap());
    /// struct Wrapper<'a, 'b>(&'b mut LoadablePluginCommand<'a>);
    /// impl<'a, 'b> std::io::Write for Wrapper<'a, 'b> {
    ///     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    ///         self.0.write_body(buf);
    ///         Ok(buf.len())
    ///     }
    ///     fn flush(&mut self) -> std::io::Result<()> {
    ///         Ok(())
    ///     }
    /// }
    /// // serialize data into command directly
    /// let request_data = serde_json::json!({
    ///     "age": 100,
    ///     "name": "name"
    /// });
    /// let mut cmd = plugin.invoke_with_capacity(command_id, subcommand_id, capacity);
    /// serde_json::to_writer(Wrapper(&mut cmd), &request_data).map_err(|err| {
    ///     trace_println!("serde error: {:?}", err);
    ///     ErrorKind::Unknown
    /// })?;
    /// let result = cmd.call()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// Notice: the shared buffer could grow to fit the request data automatically.
    pub fn invoke_with_capacity<'a>(
        &'a self,
        command_id: u32,
        subcommand_id: u32,
        capacity: usize,
    ) -> LoadablePluginCommand<'a> {
        LoadablePluginCommand::new_with_capacity(self, command_id, subcommand_id, capacity)
    }
}

impl<'a> LoadablePluginCommand<'a> {
    // use this to write request body if needed
    pub fn write_body(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }
    // same with write_body, but chainable
    pub fn chain_write_body(mut self, data: &[u8]) -> Self {
        self.write_body(data);
        self
    }
    // invoke the command, and get result from it
    pub fn call(self) -> Result<Vec<u8>> {
        let mut outlen: usize = 0;
        let mut buffer = self.buffer;
        buffer.resize(buffer.capacity(), 0); // resize to capacity first
        match unsafe {
            raw::tee_invoke_supp_plugin(
                self.plugin.uuid.as_raw_ptr(),
                self.cmd_id,
                self.sub_cmd_id,
                // convert the pointer manually, as in some platform c_char is i8
                buffer.as_mut_slice().as_mut_ptr() as *mut _,
                buffer.len(),
                &mut outlen as *mut usize,
            )
        } {
            raw::TEE_SUCCESS => {
                if outlen > buffer.len() {
                    return Err(ErrorKind::ShortBuffer.into());
                }
                buffer.resize(outlen, 0);
                Ok(buffer)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }
}

impl<'a> LoadablePluginCommand<'a> {
    fn new_with_capacity(
        plugin: &'a LoadablePlugin,
        cmd_id: u32,
        sub_cmd_id: u32,
        capacity: usize,
    ) -> Self {
        Self {
            plugin,
            cmd_id,
            sub_cmd_id,
            buffer: Vec::with_capacity(capacity),
        }
    }
}

#[cfg(test)]
pub mod test_loadable_plugin {
    extern crate std;
    use super::*;
    use core::ffi::c_char;
    use once_cell::sync::Lazy;
    use optee_utee_sys::{TEE_Result, TEE_UUID};
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::collections::HashMap;
    use std::sync::RwLock;

    static REE_RETURN_VALUES: RwLock<Lazy<HashMap<(u32, u32), Vec<u8>>>> =
        RwLock::new(Lazy::new(|| HashMap::new()));
    static REE_EXPECTED_VALUES: RwLock<Lazy<HashMap<(u32, u32), Vec<u8>>>> =
        RwLock::new(Lazy::new(|| HashMap::new()));

    fn set_ree_return_value(cmd: u32, sub_cmd: u32, value: Vec<u8>) {
        let mut values = REE_RETURN_VALUES.write().unwrap();
        let key = (cmd, sub_cmd);
        assert!(!values.contains_key(&key));
        values.insert(key, value);
    }

    fn set_ree_expected_value(cmd: u32, sub_cmd: u32, value: Vec<u8>) {
        let mut values = REE_EXPECTED_VALUES.write().unwrap();
        let key = (cmd, sub_cmd);
        assert!(!values.contains_key(&key));
        values.insert(key, value);
    }

    fn get_ree_return_value(cmd: u32, sub_cmd: u32) -> Vec<u8> {
        let values = REE_RETURN_VALUES.read().unwrap();
        let key = (cmd, sub_cmd);
        values.get(&key).unwrap().to_owned()
    }

    fn get_ree_expected_value(cmd: u32, sub_cmd: u32) -> Vec<u8> {
        let values = REE_EXPECTED_VALUES.read().unwrap();
        let key = (cmd, sub_cmd);
        values.get(&key).unwrap().to_owned()
    }

    fn generate_random_bytes(len: usize) -> Vec<u8> {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .collect()
    }

    fn generate_test_pairs(
        request_size: usize,
        response_size: usize,
    ) -> (u32, u32, Vec<u8>, Vec<u8>) {
        let cmd: u32 = rand::thread_rng().r#gen();
        let sub_cmd: u32 = rand::thread_rng().r#gen();
        let random_request: Vec<u8> = generate_random_bytes(request_size);
        let random_response: Vec<u8> = generate_random_bytes(response_size);
        (cmd, sub_cmd, random_request, random_response)
    }

    #[no_mangle]
    extern "C" fn tee_invoke_supp_plugin(
        _uuid: *const TEE_UUID,
        cmd: u32,
        sub_cmd: u32,
        buf: *mut c_char,
        len: usize,
        outlen: *mut usize,
    ) -> TEE_Result {
        // must convert buf to u8, for in some platform c_char was treated as i8
        let inbuf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
        std::println!(
            "*plugin*: receive value: {:?} length {:?}",
            inbuf,
            inbuf.len()
        );
        let expected_value = get_ree_expected_value(cmd, sub_cmd);
        assert_eq!(inbuf, expected_value.as_slice());

        let return_value = get_ree_return_value(cmd, sub_cmd);
        assert!(return_value.len() <= len);
        std::println!("*plugin*: write value '{:?}' to buffer", return_value);

        inbuf[0..return_value.len()].copy_from_slice(&return_value);
        unsafe {
            *outlen = return_value.len();
        }
        return raw::TEE_SUCCESS;
    }

    #[test]
    fn test_invoke() {
        let plugin = LoadablePlugin {
            uuid: Uuid::parse_str("7dd54ee6-a705-4e4d-8b6b-aa5024dfcd10").unwrap(),
        };
        const REQUEST_LEN: usize = 32;

        // test calling with output size less than input
        let (cmd, sub_cmd, request, exp_response) =
            generate_test_pairs(REQUEST_LEN, REQUEST_LEN / 2);
        set_ree_expected_value(cmd, sub_cmd, request.clone());
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin.invoke(cmd, sub_cmd, &request).unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);

        // test calling with output size equals to input
        let (cmd, sub_cmd, request, exp_response) = generate_test_pairs(REQUEST_LEN, REQUEST_LEN);
        set_ree_expected_value(cmd, sub_cmd, request.clone());
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin.invoke(cmd, sub_cmd, &request).unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);
    }

    #[test]
    fn test_invoke_with_capacity() {
        let plugin = LoadablePlugin {
            uuid: Uuid::parse_str("7dd54ee6-a705-4e4d-8b6b-aa5024dfcd10").unwrap(),
        };
        const RESPONSE_LEN: usize = 32;

        // test calling with output size less than input
        let (cmd, sub_cmd, request, exp_response) =
            generate_test_pairs(2 * RESPONSE_LEN, RESPONSE_LEN);
        set_ree_expected_value(cmd, sub_cmd, request.clone());
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin
            .invoke_with_capacity(cmd, sub_cmd, exp_response.len())
            .chain_write_body(&request)
            .call()
            .unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);

        // test calling with output size equals to input
        let (cmd, sub_cmd, request, exp_response) = generate_test_pairs(RESPONSE_LEN, RESPONSE_LEN);
        set_ree_expected_value(cmd, sub_cmd, request.clone());
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin
            .invoke_with_capacity(cmd, sub_cmd, exp_response.len())
            .chain_write_body(&request)
            .call()
            .unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);

        // test calling with output size greater than input
        let (cmd, sub_cmd, mut request, exp_response) =
            generate_test_pairs(RESPONSE_LEN / 2, RESPONSE_LEN);
        request.resize(exp_response.len(), 0);
        set_ree_expected_value(cmd, sub_cmd, request.clone());
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin
            .invoke_with_capacity(cmd, sub_cmd, exp_response.len())
            .chain_write_body(&request)
            .call()
            .unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);
    }
    #[test]
    fn test_invoke_with_writer() {
        let plugin = LoadablePlugin {
            uuid: Uuid::parse_str("7dd54ee6-a705-4e4d-8b6b-aa5024dfcd10").unwrap(),
        };
        // impl a writer for Command
        struct Wrapper<'a, 'b>(&'b mut LoadablePluginCommand<'a>);
        impl<'a, 'b> std::io::Write for Wrapper<'a, 'b> {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.write_body(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        // serialize data into command directly
        let test_data = serde_json::json!({
            "code": 100,
            "message": "error"
        });
        let mut exp_request = serde_json::to_vec(&test_data).unwrap();
        let buffer_len = exp_request.len() * 2;
        let (cmd, sub_cmd, _, exp_response) = generate_test_pairs(0, buffer_len);
        let mut plugin_cmd = plugin.invoke_with_capacity(cmd, sub_cmd, buffer_len);
        exp_request.resize(exp_response.len(), 0);
        set_ree_expected_value(cmd, sub_cmd, exp_request);
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        serde_json::to_writer(Wrapper(&mut plugin_cmd), &test_data).unwrap();
        let response = plugin_cmd.call().unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);
    }
    #[test]
    fn test_invoke_with_no_data() {
        let plugin = LoadablePlugin {
            uuid: Uuid::parse_str("7dd54ee6-a705-4e4d-8b6b-aa5024dfcd10").unwrap(),
        };
        const OUTPUT_LEN: usize = 50;
        let (cmd, sub_cmd, _, exp_response) = generate_test_pairs(0, OUTPUT_LEN);
        let exp_request = vec![0_u8; OUTPUT_LEN];
        set_ree_expected_value(cmd, sub_cmd, exp_request);
        set_ree_return_value(cmd, sub_cmd, exp_response.clone());
        let response = plugin
            .invoke_with_capacity(cmd, sub_cmd, OUTPUT_LEN)
            .call()
            .unwrap();
        std::println!("*TA*: response is {:?}", response);
        assert_eq!(response, exp_response);
    }
}
