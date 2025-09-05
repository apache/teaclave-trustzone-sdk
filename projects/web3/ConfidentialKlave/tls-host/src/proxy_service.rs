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

use anyhow::Result;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::ta_tls_command::{TaTlsRequest, TaTlsRequestCommand, TaTlsResponse};

pub struct ProxyRequest {
    pub request: TaTlsRequest,
    pub xsender: UnboundedSender<Result<TaTlsResponse>>,
}

pub struct ProxyService {
    sender: UnboundedSender<ProxyRequest>,
    session_id: u32,
}
impl ProxyService {
    pub fn new(sender: UnboundedSender<ProxyRequest>, session_id: u32) -> Self {
        ProxyService { sender, session_id }
    }

    pub async fn new_tls_session(&mut self) -> Result<()> {
        let ta_tls_request = TaTlsRequest {
            cmd: TaTlsRequestCommand::NewTlsSession,
            session_id: self.session_id,
            buffer: Vec::new(),
        };
        self.handle_request(ta_tls_request).await?;
        Ok(())
    }

    pub async fn close_tls_session(&mut self) -> Result<()> {
        let ta_tls_request = TaTlsRequest {
            cmd: TaTlsRequestCommand::CloseTlsSession,
            session_id: self.session_id,
            buffer: Vec::new(),
        };
        self.handle_request(ta_tls_request).await?;
        Ok(())
    }

    pub async fn send_to_ta(&mut self, buffer: Vec<u8>) -> Result<()> {
        let ta_tls_request = TaTlsRequest {
            cmd: TaTlsRequestCommand::SendToTa,
            session_id: self.session_id,
            buffer,
        };
        self.handle_request(ta_tls_request).await?;
        Ok(())
    }

    pub async fn receive_from_ta(&mut self) -> Result<Vec<u8>> {
        let ta_tls_request = TaTlsRequest {
            cmd: TaTlsRequestCommand::ReceiveFromTa,
            session_id: self.session_id,
            buffer: Vec::new(),
        };
        let response = self.handle_request(ta_tls_request).await?;
        Ok(response.buffer)
    }

    async fn handle_request(&mut self, request: TaTlsRequest) -> Result<TaTlsResponse> {
        let (xsender, mut xreceiver) = unbounded_channel();
        println!("x_channel created");
        let proxy_request = ProxyRequest { request, xsender };
        self.sender
            .send(proxy_request)
            .map_err(|e| anyhow::anyhow!("Error sending proxy request: {}", e))?;
        xreceiver
            .recv()
            .await
            .unwrap_or_else(|| anyhow::bail!("Error receiving proxy response"))
    }
}
