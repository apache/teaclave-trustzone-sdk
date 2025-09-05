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

use constants;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn main() -> std::io::Result<()> {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut buffer = File::create(out.join("user_ta_header.rs"))?;
    buffer.write_all(include_bytes!("ta_static.rs"))?;

    let tee_uuid = Uuid::parse_str(constants::TLS_UUID).unwrap();
    let (time_low, time_mid, time_hi_and_version, clock_seq_and_node) = tee_uuid.as_fields();

    write!(buffer, "\n")?;
    write!(
        buffer,
        "const TA_UUID: optee_utee_sys::TEE_UUID = optee_utee_sys::TEE_UUID {{
    timeLow: {:#x},
    timeMid: {:#x},
    timeHiAndVersion: {:#x},
    clockSeqAndNode: {:#x?},
}};",
        time_low, time_mid, time_hi_and_version, clock_seq_and_node
    )?;
    let optee_os_dir = env::var("OPTEE_OS_DIR").unwrap_or("../../../optee/optee_os".to_string());
    let search_path = match env::var("ARCH") {
        Ok(ref v) if v == "arm" => {
            File::create(out.join("ta.lds"))?.write_all(include_bytes!("ta_arm.lds"))?;
            Path::new(&optee_os_dir).join("out/arm/export-ta_arm32/lib")
        }
        _ => {
            File::create(out.join("ta.lds"))?.write_all(include_bytes!("ta_aarch64.lds"))?;
            Path::new(&optee_os_dir).join("out/arm/export-ta_arm64/lib")
        }
    };
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=ta.lds");

    println!("cargo:rustc-link-search={}", search_path.display());
    println!("cargo:rustc-link-lib=static=utee");
    Ok(())
}
