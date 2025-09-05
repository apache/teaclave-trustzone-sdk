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

use optee_teec::{Context, ErrorKind, Operation, Session, Uuid};
use optee_teec::{ParamNone, ParamTmpRef, ParamType, ParamValue};
use proto::{Command, UUID};
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

const MAX_PAYLOAD: u16 = 16384 + 2048;
const HEADER_SIZE: u16 = 1 + 2 + 2;
pub const MAX_WIRE_SIZE: usize = (MAX_PAYLOAD + HEADER_SIZE) as usize;

fn main() -> optee_teec::Result<()> {
    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID)?;
    let mut ta_session = ctx.open_session(uuid)?;

    let mut session_id: u32 = 0;
    println!("listening");
    let listener = TcpListener::bind("0.0.0.0:4433").map_err(|e| {
        eprintln!("Failed to bind TCP listener: {}", e);
        ErrorKind::BadParameters
    })?;

    for stream in listener.incoming() {
        session_id += 1;
        let stream = stream.map_err(|e| {
            eprintln!("Failed to accept TCP connection: {}", e);
            ErrorKind::Communication
        })?;
        handle_client(&mut ta_session, session_id, stream)?;
    }

    println!("Success");
    Ok(())
}

fn new_tls_session(ta_session: &mut Session, session_id: u32) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    ta_session.invoke_command(Command::NewTlsSession as u32, &mut operation)?;
    Ok(())
}

fn close_tls_session(ta_session: &mut Session, session_id: u32) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);
    ta_session.invoke_command(Command::CloseTlsSession as u32, &mut operation)?;
    Ok(())
}

fn do_tls_read(
    ta_session: &mut Session,
    session_id: u32,
    buf: &mut [u8],
) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let p1 = ParamTmpRef::new_input(buf);
    let mut operation = Operation::new(0, p0, p1, ParamNone, ParamNone);
    ta_session.invoke_command(Command::DoTlsRead as u32, &mut operation)?;
    Ok(())
}

fn do_tls_write(
    ta_session: &mut Session,
    session_id: u32,
    buf: &mut [u8],
) -> optee_teec::Result<usize> {
    let p0 = ParamValue::new(session_id, 0, ParamType::ValueInput);
    let p1 = ParamTmpRef::new_output(buf);
    let p2 = ParamValue::new(0, 0, ParamType::ValueOutput);
    let mut operation = Operation::new(0, p0, p1, p2, ParamNone);
    ta_session.invoke_command(Command::DoTlsWrite as u32, &mut operation)?;
    Ok(operation.parameters().2.a() as usize)
}

fn handle_client(
    ta_session: &mut Session,
    session_id: u32,
    mut stream: TcpStream,
) -> optee_teec::Result<()> {
    println!("new session");
    new_tls_session(ta_session, session_id)?;
    loop {
        let mut buf = [0u8; MAX_WIRE_SIZE];
        println!("stream read");
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => {
                println!("close session");
                close_tls_session(ta_session, session_id)?;
                break;
            }
            Ok(n) => {
                println!("read bytes: {}", n);
                do_tls_read(ta_session, session_id, &mut buf[..n])?
            }
        }

        let n = do_tls_write(ta_session, session_id, &mut buf)?;
        println!("stream write n: {}", n);
        let res = stream.write_all(&buf[..n]);
        if res.is_err() {
            println!("close session");
            close_tls_session(ta_session, session_id)?;
            break;
        }
    }

    Ok(())
}
