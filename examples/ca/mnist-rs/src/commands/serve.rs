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

use clap::Parser;
use image::EncodableLayout;
use proto::mnist::{IMAGE_HEIGHT, IMAGE_SIZE, IMAGE_WIDTH};
use std::io::Cursor;
use tiny_http::{Request, Response, Server};

#[derive(Parser, Debug)]
pub struct Args {
    /// The path of the model.
    #[arg(short, long)]
    model: String,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

type HttpResponse = Response<Cursor<Vec<u8>>>;

pub fn execute(args: &Args) -> anyhow::Result<()> {
    let model_path = std::path::absolute(&args.model)?;
    println!("Load model from \"{}\"", model_path.display());
    let record = std::fs::read(&model_path)?;
    let ctx = optee_teec::Context::new()?;
    let mut caller = crate::tee::InferenceTaConnector::new(&ctx, &record)?;

    let addr = format!("0.0.0.0:{}", args.port);
    println!("Server runs on: {addr}");

    let server = Server::http(&addr)
        .map_err(|err| anyhow::Error::msg(format!("cannot start server: {err:?}")))?;

    loop {
        let mut request = server.recv()?;
        let response = match handle(&mut caller, &mut request) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("unexpected error: {err:#?}");
                Response::from_string("Internal Error").with_status_code(500)
            }
        };
        request.respond(response)?;
    }
}

fn handle(
    caller: &mut crate::tee::InferenceTaConnector,
    request: &mut Request,
) -> anyhow::Result<HttpResponse> {
    if request.method().ne(&tiny_http::Method::Post) {
        return Ok(Response::from_string("Invalid Request Method").with_status_code(400));
    }

    match request.url() {
        "/inference/image" => handle_image(caller, request),
        "/inference/binary" => handle_binary(caller, request),
        _ => Ok(Response::from_string("Not Found").with_status_code(404)),
    }
}

fn handle_image(
    caller: &mut crate::tee::InferenceTaConnector,
    request: &mut Request,
) -> anyhow::Result<HttpResponse> {
    let mut data = Vec::with_capacity(IMAGE_SIZE);
    request.as_reader().read_to_end(&mut data)?;
    let img = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?
        .to_luma8();
    if img.width() as usize != IMAGE_WIDTH || img.height() as usize != IMAGE_HEIGHT {
        return Ok(Response::from_string("Invalid Image").with_status_code(400));
    }
    let result = handle_infer(caller, img.as_bytes())?;

    println!("Performing Inference with Image, Result is {result}");
    Ok(Response::from_data(result.to_string()))
}

fn handle_binary(
    caller: &mut crate::tee::InferenceTaConnector,
    request: &mut Request,
) -> anyhow::Result<HttpResponse> {
    let mut data = Vec::with_capacity(IMAGE_SIZE);
    request.as_reader().read_to_end(&mut data)?;
    if data.len() != IMAGE_SIZE {
        return Ok(Response::from_string("Invalid Tensor").with_status_code(400));
    }

    let result = handle_infer(caller, &data)?;
    println!("Performing Inference with Binary, Result is {result}");
    Ok(Response::from_data(result.to_string()))
}

fn handle_infer(caller: &mut crate::tee::InferenceTaConnector, image: &[u8]) -> anyhow::Result<u8> {
    let result = caller.infer_batch(bytemuck::cast_slice(image))?;
    Ok(result[0])
}
