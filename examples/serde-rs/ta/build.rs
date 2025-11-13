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

use optee_utee_build::{Error, RustEdition, TaConfig};

fn main() -> Result<(), Error> {
    let ta_config = TaConfig::new_default_with_cargo_env(proto::UUID)?
        .ta_stack_size(4 * 1024)
        .ta_data_size(64 * 1024);
    optee_utee_build::build(RustEdition::Before2024, ta_config)
}
