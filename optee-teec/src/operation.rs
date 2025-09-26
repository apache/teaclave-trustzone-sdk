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

use crate::{raw, Param, ParamTypes};
use std::{marker::PhantomData, mem};

/// This type defines the payload of either an open session operation or an
/// invoke command operation. It is also used for cancellation of operations,
/// which may be desirable even if no payload is passed.
pub struct Operation<A, B, C, D> {
    raw: raw::TEEC_Operation,
    phantom0: PhantomData<A>,
    phantom1: PhantomData<B>,
    phantom2: PhantomData<C>,
    phantom3: PhantomData<D>,
}

impl<A: Param, B: Param, C: Param, D: Param> Operation<A, B, C, D> {
    pub fn new(started: u32, mut p0: A, mut p1: B, mut p2: C, mut p3: D) -> Operation<A, B, C, D> {
        let mut raw_op: raw::TEEC_Operation = unsafe { mem::zeroed() };
        raw_op.started = started;
        raw_op.paramTypes = ParamTypes::new(
            p0.param_type(),
            p1.param_type(),
            p2.param_type(),
            p3.param_type(),
        )
        .into();
        raw_op.params = [p0.to_raw(), p1.to_raw(), p2.to_raw(), p3.to_raw()];
        Operation {
            raw: raw_op,
            phantom0: PhantomData,
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }

    pub(crate) fn as_mut_raw_ptr(&mut self) -> *mut raw::TEEC_Operation {
        &mut self.raw
    }

    pub fn parameters(&self) -> (A, B, C, D) {
        let (f0, f1, f2, f3) = ParamTypes::from(self.raw.paramTypes).into_flags();
        (
            A::from_raw(self.raw.params[0], f0),
            B::from_raw(self.raw.params[1], f1),
            C::from_raw(self.raw.params[2], f2),
            D::from_raw(self.raw.params[3], f3),
        )
    }
}
