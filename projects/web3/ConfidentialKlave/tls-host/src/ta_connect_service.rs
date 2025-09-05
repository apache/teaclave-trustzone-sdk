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

use anyhow::anyhow;
use constants::TLS_UUID;
use optee_teec::{Context, Operation, Session, Uuid};
use optee_teec::{ParamNone, ParamTmpRef, ParamType, ParamValue};
use proto::TaCommand;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::proxy_service::ProxyRequest;
use crate::ta_tls_command::{TaTlsRequestCommand, TaTlsResponse};

const MAX_PAYLOAD: u16 = 16384 + 2048;
const HEADER_SIZE: u16 = 1 + 2 + 2;
pub const MAX_WIRE_SIZE: usize = (MAX_PAYLOAD + HEADER_SIZE) as usize;

pub struct TaConnectService {}
impl TaConnectService {
    pub fn start(
        tee_cert: Vec<u8>,
        mut receiver: UnboundedReceiver<ProxyRequest>,
    ) -> optee_teec::Result<()> {
        println!("TaConnectService started");
        let mut ctx = Context::new()?;
        println!("Context created");
        let uuid = Uuid::parse_str(TLS_UUID).unwrap();
        let mut ta_session = ctx.open_session(uuid)?;
        println!("Session opened");

        let session_id: u32 = 0;
        let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);

        let p1 = ParamTmpRef::new_input(&tee_cert);
        let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);
        ta_session.invoke_command(TaCommand::StartServer as u32, &mut operation)?;
        println!("TA invoked StartServer");
        loop {
            let request = match receiver.blocking_recv() {
                Some(request) => {
                    println!(
                        "TaConnectService received request, cmd: {:?}",
                        request.request.cmd
                    );
                    request
                }
                None => continue,
            };
            let ta_tls_request = request.request;
            let xsender = request.xsender;
            let response = match ta_tls_request.cmd {
                TaTlsRequestCommand::NewTlsSession => {
                    match new_tls_session(&mut ta_session, ta_tls_request.session_id) {
                        Ok(_) => Ok(TaTlsResponse { buffer: Vec::new() }),
                        Err(e) => Err(anyhow!(
                            "Error invoking new_tls_session, session id: {}, error: {}",
                            ta_tls_request.session_id,
                            e
                        )),
                    }
                }
                TaTlsRequestCommand::SendToTa => {
                    match tls_send_to_ta(
                        &mut ta_session,
                        ta_tls_request.session_id,
                        &ta_tls_request.buffer,
                    ) {
                        Ok(_) => Ok(TaTlsResponse { buffer: Vec::new() }),
                        Err(e) => Err(anyhow!(
                            "Error invoking tls_send_to_ta, session id: {}, error: {}",
                            ta_tls_request.session_id,
                            e
                        )),
                    }
                }
                TaTlsRequestCommand::ReceiveFromTa => {
                    match tls_receive_from_ta(&mut ta_session, ta_tls_request.session_id) {
                        Ok(buf) => Ok(TaTlsResponse { buffer: buf }),
                        Err(e) => Err(anyhow!(
                            "Error invoking tls_receive_from_ta, session id: {}, error: {}",
                            ta_tls_request.session_id,
                            e
                        )),
                    }
                }
                TaTlsRequestCommand::CloseTlsSession => {
                    match close_tls_session(&mut ta_session, ta_tls_request.session_id) {
                        Ok(_) => Ok(TaTlsResponse { buffer: Vec::new() }),
                        Err(e) => Err(anyhow!(
                            "Error invoking close_tls_session, session id: {}, error: {}",
                            ta_tls_request.session_id,
                            e
                        )),
                    }
                }
            };
            match xsender.send(response) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error sending response: {}", e);
                }
            }
        }
    }
}

fn new_tls_session(ta_session: &mut Session, session_id: u32) -> optee_teec::Result<Vec<u8>> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    ta_session.invoke_command(TaCommand::NewTlsSession as u32, &mut operation)?;
    println!("after invoking TA: new_tls_session");
    Ok(Vec::new())
}

fn tls_send_to_ta(
    ta_session: &mut Session,
    session_id: u32,
    buf: &[u8],
) -> optee_teec::Result<Vec<u8>> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let p1 = ParamTmpRef::new_input(buf);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);
    ta_session.invoke_command(TaCommand::DoTlsRead as u32, &mut operation)?;
    println!("after invoking TA: do_tls_read");
    Ok(Vec::new())
}

fn tls_receive_from_ta(ta_session: &mut Session, session_id: u32) -> optee_teec::Result<Vec<u8>> {
    let mut buf = [0u8; MAX_WIRE_SIZE];
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let p1 = ParamTmpRef::new_output(&mut buf);
    let p2 = ParamValue::new(0, 0, ParamType::ValueOutput);
    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);
    ta_session.invoke_command(TaCommand::DoTlsWrite as u32, &mut operation)?;
    println!("after invoking TA: do_tls_write");
    let buffer_size = operation.parameters().2.a() as usize;
    Ok(buf[..buffer_size].to_vec())
}

fn close_tls_session(ta_session: &mut Session, session_id: u32) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    ta_session.invoke_command(TaCommand::CloseTlsSession as u32, &mut operation)?;
    println!("after invoking TA: close session");
    Ok(())
}
