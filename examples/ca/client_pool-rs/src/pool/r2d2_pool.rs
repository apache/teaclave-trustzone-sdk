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

use super::{
    connection::{tee_wait, Connection},
    Args,
};
use optee_teec::{Context, Uuid};
use std::{
    sync::{atomic, Arc},
    thread,
    time::Duration,
};

struct Manager {
    ctx: Context,
    uuid: Uuid,
}

impl r2d2::ManageConnection for Manager {
    type Error = optee_teec::Error;
    type Connection = Connection;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Connection::new(&self.ctx, self.uuid.clone())
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.valid()
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.valid().is_err()
    }
}

fn new_pool(args: &Args) -> Result<r2d2::Pool<Manager>, anyhow::Error> {
    let manager = Manager {
        ctx: Context::new()?,
        uuid: Uuid::parse_str(proto::client_pool::UUID)?,
    };

    Ok(r2d2::Pool::builder()
        .max_size(args.pool_capacity)
        .build(manager)?)
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let pool = new_pool(&args)?;
    let finish_counter = Arc::new(atomic::AtomicU64::new(0));

    for i in 0..args.concurrency {
        let pool = pool.clone();
        let finish_counter = finish_counter.clone();
        let ta_wait_timeout = args.ta_wait_timeout;
        thread::spawn(move || -> anyhow::Result<()> {
            let mut session = pool.get()?;
            tee_wait(&mut session, ta_wait_timeout)?;
            finish_counter.fetch_add(1, atomic::Ordering::Relaxed);
            println!(
                "r2d2: {}: {} finish",
                i,
                hex::encode_upper(session.identity())
            );
            Ok(())
        });
    }

    thread::sleep(Duration::from_millis(args.execution_timeout as u64));
    println!(
        "r2d2: total tasks: {}, total finish: {}",
        args.concurrency,
        finish_counter.load(atomic::Ordering::Relaxed)
    );

    Ok(())
}
