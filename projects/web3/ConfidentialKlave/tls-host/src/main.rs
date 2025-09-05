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

mod cli;
mod ta_connect_service;
use ta_connect_service::*;
mod proxy_service;
use proxy_service::*;
mod ta_tls_command;

use std::fs;
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use anyhow::Result;

const MAX_PAYLOAD: u16 = 16384 + 2048;
const HEADER_SIZE: u16 = 1 + 2 + 2;
pub const MAX_WIRE_SIZE: usize = (MAX_PAYLOAD + HEADER_SIZE) as usize;

#[tokio::main]
async fn main() -> optee_teec::Result<()> {
    let args = cli::Opt::from_args();
    let tee_cert = match args.command {
        cli::Command::StartServer(opt) => {
            let tee_cert = match fs::read(&opt.tee_cert) {
                Ok(file) => file,
                Err(e) => {
                    println!("Error reading tee_cert: {}", e);
                    return Ok(());
                }
            };
            tee_cert
        }
    };

    let (sender, receiver) = unbounded_channel();
    std::thread::spawn(move || match TaConnectService::start(tee_cert, receiver) {
        Ok(_) => {}
        Err(e) => {
            println!("Error starting TaConnectService: {}", e);
        }
    });
    println!("ta_connect_service started");

    // listen for incoming connections
    println!("listening");
    let listener = TcpListener::bind("0.0.0.0:4433").await.unwrap();
    let mut session_id: u32 = 0;
    assert!(!sender.is_closed());
    loop {
        let sender_dup = sender.clone();
        match listener.accept().await {
            Ok((stream, _)) => {
                session_id += 1;
                println!("new client, session_id: {}", session_id);
                tokio::spawn(async move {
                    match handle_client(sender_dup, session_id, stream).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Error handling client: {}", e);
                        }
                    }
                });
            }
            Err(e) => eprintln!("Error accepting connection: {:?}", e),
        }
    }
}

async fn handle_client(
    sender: UnboundedSender<ProxyRequest>,
    session_id: u32,
    mut stream: TcpStream,
) -> Result<()> {
    let mut proxy_service = ProxyService::new(sender, session_id);
    proxy_service.new_tls_session().await?;
    println!("new session: {}", session_id);
    loop {
        let mut buf = [0u8; MAX_WIRE_SIZE];
        match stream.read(&mut buf).await {
            Ok(0) | Err(_) => {
                proxy_service.close_tls_session().await?;
                println!("close session: {}", session_id);
                break Ok(());
            }
            Ok(n) => {
                println!("read bytes: {}, session_id: {}", n, session_id);
                proxy_service.send_to_ta(buf[..n].to_vec()).await?;
                println!("do_tls_read finished");
            }
        }
        let buf = proxy_service.receive_from_ta().await?;
        println!(
            "do_tls_write finished, session_id: {}, buf len: {}",
            session_id,
            buf.len()
        );
        let res = stream.write_all(&buf).await;
        if res.is_err() {
            println!("error: {:?}, close tls session: {}", res, session_id);
            proxy_service.close_tls_session().await?;
            println!("close stream finished");
            break Ok(());
        }
    }
}
