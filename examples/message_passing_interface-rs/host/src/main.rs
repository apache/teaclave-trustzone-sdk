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

use optee_teec::{
    Context, ErrorKind, Operation, ParamNone, ParamTmpRef, ParamType, ParamValue, Uuid,
};

type Result<T> = optee_teec::Result<T>;

pub struct EnclaveClient {
    uuid: String,
    context: optee_teec::Context,
    buffer: Vec<u8>,
}

impl EnclaveClient {
    pub fn open(url: &str) -> Result<Self> {
        let url = url::Url::parse(url).map_err(|e| {
            eprintln!("Invalid URL: {}, error: {}", url, e);
            ErrorKind::BadParameters
        })?;
        match url.scheme() {
            "trustzone-enclave" => Self::open_uuid(url.host_str().ok_or_else(|| {
                eprintln!("Missing host in URL");
                ErrorKind::BadParameters
            })?),
            _ => unimplemented!(),
        }
    }

    fn open_uuid(uuid: &str) -> Result<Self> {
        let context = Context::new()?;
        Ok(Self {
            uuid: uuid.to_string(),
            context,
            buffer: vec![0; 128],
        })
    }

    pub fn invoke(&mut self, input: &proto::EnclaveInput) -> Result<proto::EnclaveOutput> {
        let command_id = input.command as u32;
        let mut serialized_input = serde_json::to_vec(input).map_err(|e| {
            eprintln!("Failed to serialize input: {}", e);
            ErrorKind::BadParameters
        })?;

        let p0 = ParamTmpRef::new_input(serialized_input.as_mut_slice());
        let p1 = ParamTmpRef::new_output(&mut self.buffer);
        let p2 = ParamValue::new(0, 0, ParamType::ValueInout);

        let mut operation = Operation::new(0, p0, p1, p2, ParamNone);

        let uuid = Uuid::parse_str(&self.uuid)?;
        let mut session = self.context.open_session(uuid)?;
        session.invoke_command(command_id, &mut operation)?;
        let len = operation.parameters().2.a() as usize;

        let output: proto::EnclaveOutput =
            serde_json::from_slice(&self.buffer[0..len]).map_err(|e| {
                eprintln!("Failed to deserialize output: {}", e);
                ErrorKind::BadParameters
            })?;
        Ok(output)
    }
}

fn main() -> optee_teec::Result<()> {
    let url = format!("trustzone-enclave://{}", proto::UUID);
    let mut enclave = EnclaveClient::open(&url)?;
    let input = proto::EnclaveInput {
        command: proto::Command::Hello,
        message: String::from("World!"),
    };
    let output = enclave.invoke(&input)?;
    println!("{:?}", output);

    Ok(())
}
