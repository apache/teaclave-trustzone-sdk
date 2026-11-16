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

use optee_teec::{Context, ErrorKind, Operation, ParamNone, ParamTmpRef, Session, Uuid};
use proto::mnist::{inference, train, Image};

const MAX_OUTPUT_SERIALIZE_SIZE: usize = 1024;
const MAX_MODEL_RECORD_SIZE: usize = 10 * 1024 * 1024;

pub struct TrainerTaConnector {
    sess: Session,
}

impl TrainerTaConnector {
    pub fn new(ctx: &Context, learning_rate: f64) -> optee_teec::Result<Self> {
        let bytes = learning_rate.to_le_bytes();
        let uuid = Uuid::parse_str(train::UUID).map_err(|err| {
            println!("parse uuid \"{}\" failed due to: {:?}", train::UUID, err);
            ErrorKind::BadParameters
        })?;
        let mut op = Operation::new(
            0,
            ParamTmpRef::new_input(bytes.as_slice()),
            ParamNone,
            ParamNone,
            ParamNone,
        );

        Ok(Self {
            sess: ctx.open_session_with_operation(uuid, &mut op)?,
        })
    }
    pub fn train(&mut self, images: &[Image], labels: &[u8]) -> optee_teec::Result<train::Output> {
        let mut buffer = vec![0_u8; MAX_OUTPUT_SERIALIZE_SIZE];
        let images = bytemuck::cast_slice(images);
        let size = {
            let mut op = Operation::new(
                0,
                ParamTmpRef::new_input(images),
                ParamTmpRef::new_input(labels),
                ParamTmpRef::new_output(&mut buffer),
                ParamNone,
            );
            self.sess
                .invoke_command(train::Command::Train as u32, &mut op)?;
            op.parameters().2.updated_size()
        };
        let result = serde_json::from_slice(&buffer[0..size]).map_err(|err| {
            println!("proto error: {err:?}");
            ErrorKind::BadFormat
        })?;
        Ok(result)
    }
    pub fn valid(&mut self, images: &[Image], labels: &[u8]) -> optee_teec::Result<train::Output> {
        let mut buffer = vec![0_u8; MAX_OUTPUT_SERIALIZE_SIZE];
        let images = bytemuck::cast_slice(images);
        let size = {
            let mut op = Operation::new(
                0,
                ParamTmpRef::new_input(images),
                ParamTmpRef::new_input(labels),
                ParamTmpRef::new_output(&mut buffer),
                ParamNone,
            );
            self.sess
                .invoke_command(train::Command::Valid as u32, &mut op)?;
            op.parameters().2.updated_size()
        };
        let result = serde_json::from_slice(&buffer[0..size]).map_err(|err| {
            println!("proto error: {err:?}");
            ErrorKind::BadFormat
        })?;
        Ok(result)
    }

    pub fn export(&mut self) -> optee_teec::Result<Vec<u8>> {
        let mut buffer = vec![0_u8; MAX_MODEL_RECORD_SIZE];
        let size = {
            let mut op = Operation::new(
                0,
                ParamTmpRef::new_output(&mut buffer),
                ParamNone,
                ParamNone,
                ParamNone,
            );
            self.sess
                .invoke_command(train::Command::Export as u32, &mut op)?;
            op.parameters().0.updated_size()
        };
        buffer.resize(size, 0);
        Ok(buffer)
    }
}

pub struct InferenceTaConnector {
    sess: Session,
}

impl InferenceTaConnector {
    pub fn new(ctx: &Context, record: &[u8]) -> optee_teec::Result<Self> {
        let uuid = Uuid::parse_str(inference::UUID).map_err(|err| {
            println!(
                "parse uuid \"{}\" failed due to: {:?}",
                inference::UUID,
                err
            );
            ErrorKind::BadParameters
        })?;
        let mut op = Operation::new(
            0,
            ParamTmpRef::new_input(record),
            ParamNone,
            ParamNone,
            ParamNone,
        );

        Ok(Self {
            sess: ctx.open_session_with_operation(uuid, &mut op)?,
        })
    }
    pub fn infer_batch(&mut self, images: &[Image]) -> optee_teec::Result<Vec<u8>> {
        let mut output = vec![0_u8; images.len()];
        let size = {
            let mut op = Operation::new(
                0,
                ParamTmpRef::new_input(bytemuck::cast_slice(images)),
                ParamTmpRef::new_output(&mut output),
                ParamNone,
                ParamNone,
            );
            self.sess.invoke_command(0, &mut op)?;
            op.parameters().1.updated_size()
        };

        if output.len() != size {
            println!("mismatch response, want {}, got {}", size, output.len());
            return Err(ErrorKind::Generic.into());
        }
        Ok(output)
    }
}
