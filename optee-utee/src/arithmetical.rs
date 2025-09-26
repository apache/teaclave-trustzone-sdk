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

use crate::{Error, Result};
use optee_utee_sys as raw;
use core::{cmp::max, fmt};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub type BigIntUnit = u32;
pub type BigIntFMMUnit = u32;
pub type BigIntFMMContextUnit = u32;

pub struct BigInt(Vec<BigIntUnit>);

impl BigInt {
    pub fn data_ptr(&self) -> *const u32 {
        self.0.as_ptr()
    }

    // size represents BigInt bits
    pub fn size_in_u32(size: u32) -> u32 {
        ((size + 31) / 32) + 2
    }

    pub fn new(bits: u32) -> Self {
        let size: usize = Self::size_in_u32(bits) as usize;
        let mut tmp_vec: Vec<BigIntUnit> = vec![0; size];
        unsafe { raw::TEE_BigIntInit(tmp_vec.as_mut_ptr(), size) };
        Self(tmp_vec)
    }

    pub fn convert_from_octet_string(&mut self, buffer: &[u8], sign: i32) -> Result<()> {
        match unsafe {
            raw::TEE_BigIntConvertFromOctetString(
                self.0.as_mut_ptr(),
                buffer.as_ptr(),
                buffer.len(),
                sign,
            )
        } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    pub fn convert_to_octet_string(&self) -> Result<Vec<u8>> {
        let mut buffer_size: usize = (self.0.len() - 2) * 4;
        let mut tmp_vec = vec![0u8; buffer_size];
        match unsafe {
            raw::TEE_BigIntConvertToOctetString(
                tmp_vec.as_mut_ptr(),
                &mut buffer_size,
                self.data_ptr(),
            )
        } {
            raw::TEE_SUCCESS => {
                tmp_vec.truncate(buffer_size);
                Ok(tmp_vec)
            }
            code => Err(Error::from_raw_error(code)),
        }
    }

    pub fn convert_from_s32(&mut self, short_val: i32) {
        unsafe { raw::TEE_BigIntConvertFromS32(self.0.as_mut_ptr(), short_val) };
    }

    pub fn convert_to_s32(&self) -> Result<i32> {
        let mut short_val: i32 = 0;
        match unsafe { raw::TEE_BigIntConvertToS32(&mut short_val as _, self.data_ptr()) } {
            raw::TEE_SUCCESS => Ok(short_val),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /* return negative number if self < target,
    0 if self == target
    positive number if self > target*/
    pub fn compare_big_int(&self, target: &Self) -> i32 {
        unsafe { raw::TEE_BigIntCmp(self.data_ptr(), target.data_ptr()) }
    }

    pub fn compare_s32(&self, target: i32) -> i32 {
        unsafe { raw::TEE_BigIntCmpS32(self.data_ptr(), target) }
    }

    pub fn shift_right(&mut self, op: &Self, bits: usize) {
        // Should return a BigInt, while its size is based on the abs function which is missed
        // right now
        unsafe { raw::TEE_BigIntShiftRight(self.0.as_mut_ptr(), op.data_ptr(), bits) };
    }

    pub fn get_bit(&self, bit_index: u32) -> bool {
        unsafe { raw::TEE_BigIntGetBit(self.data_ptr(), bit_index) }
    }

    pub fn get_bit_count(&self) -> u32 {
        unsafe { raw::TEE_BigIntGetBitCount(self.data_ptr()) }
    }

    pub fn add(op1: &Self, op2: &Self) -> Self {
        let bits = max(Self::get_bit_count(op1), Self::get_bit_count(op2)) + 1;
        let mut res = Self::new(bits);
        unsafe { raw::TEE_BigIntAdd(res.0.as_mut_ptr(), op1.data_ptr(), op2.data_ptr()) };
        res
    }

    pub fn sub(op1: &Self, op2: &Self) -> Self {
        let bits = max(Self::get_bit_count(op1), Self::get_bit_count(op2)) + 1;
        let mut res = Self::new(bits);
        unsafe { raw::TEE_BigIntSub(res.0.as_mut_ptr(), op1.data_ptr(), op2.data_ptr()) };
        res
    }

    pub fn neg(op: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(op));
        unsafe { raw::TEE_BigIntNeg(res.0.as_mut_ptr(), op.data_ptr()) };
        res
    }

    pub fn multiply(op1: &Self, op2: &Self) -> Self {
        let bits = Self::get_bit_count(op1) + Self::get_bit_count(op2);
        let mut res = Self::new(bits);
        unsafe { raw::TEE_BigIntMul(res.0.as_mut_ptr(), op1.data_ptr(), op2.data_ptr()) };
        res
    }

    pub fn square(op: &Self) -> Self {
        let mut res = Self::new(2 * Self::get_bit_count(op));
        unsafe { raw::TEE_BigIntSquare(res.0.as_mut_ptr(), op.data_ptr()) };
        res
    }

    // document defines wrong size for result quotient
    pub fn divide(op1: &Self, op2: &Self) -> (Self, Self) {
        let q_bits = match op1.compare_big_int(op2) {
            d if d >= 0 => max(1, Self::get_bit_count(op1) - Self::get_bit_count(op2)),
            _ => 0,
        };
        let r_bits = Self::get_bit_count(op2);
        let mut quotient = Self::new(q_bits);
        let mut remainder = Self::new(r_bits);

        unsafe {
            raw::TEE_BigIntDiv(
                quotient.0.as_mut_ptr(),
                remainder.0.as_mut_ptr(),
                op1.data_ptr(),
                op2.data_ptr(),
            )
        };
        (quotient, remainder)
    }

    pub fn module(op: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe { raw::TEE_BigIntMod(res.0.as_mut_ptr(), op.data_ptr(), n.data_ptr()) };
        res
    }

    pub fn add_mod(op1: &Self, op2: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe {
            raw::TEE_BigIntAddMod(
                res.0.as_mut_ptr(),
                op1.data_ptr(),
                op2.data_ptr(),
                n.data_ptr(),
            )
        };
        res
    }

    pub fn sub_mod(op1: &Self, op2: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe {
            raw::TEE_BigIntSubMod(
                res.0.as_mut_ptr(),
                op1.data_ptr(),
                op2.data_ptr(),
                n.data_ptr(),
            )
        };
        res
    }

    pub fn mul_mod(op1: &Self, op2: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe {
            raw::TEE_BigIntMulMod(
                res.0.as_mut_ptr(),
                op1.data_ptr(),
                op2.data_ptr(),
                n.data_ptr(),
            )
        };
        res
    }

    pub fn square_mod(op: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe { raw::TEE_BigIntSquareMod(res.0.as_mut_ptr(), op.data_ptr(), n.data_ptr()) };
        res
    }

    pub fn inv_mod(op: &Self, n: &Self) -> Self {
        let mut res = Self::new(Self::get_bit_count(n));
        unsafe { raw::TEE_BigIntInvMod(res.0.as_mut_ptr(), op.data_ptr(), n.data_ptr()) };
        res
    }

    pub fn relative_prime(op1: &Self, op2: &Self) -> bool {
        unsafe { raw::TEE_BigIntRelativePrime(op1.data_ptr(), op2.data_ptr()) }
    }

    /* pub fn compute_extended_gcd(op1: &Self, op2: &Self) -> (Self, Self, Self)
     * This function is implemented in OP-TEE, while the output size needs to be calculated based
     * on the missing function TEE_BigIntAbs, so we do not port it yet.*/

    pub fn is_probable_prime(&self, confidence_level: u32) -> i32 {
        unsafe { raw::TEE_BigIntIsProbablePrime(self.data_ptr(), confidence_level) }
    }

    pub fn convert_from_big_int_fmm(
        &mut self,
        src: &BigIntFMM,
        n: &BigInt,
        context: BigIntFMMContext,
    ) {
        unsafe {
            raw::TEE_BigIntConvertToFMM(
                self.0.as_mut_ptr(),
                src.data_ptr(),
                n.data_ptr(),
                context.data_ptr(),
            )
        };
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

pub struct BigIntFMMContext(Vec<BigIntFMMContextUnit>);

impl BigIntFMMContext {
    pub fn data_ptr(&self) -> *const u32 {
        self.0.as_ptr()
    }

    fn size_in_u32(size: usize) -> usize {
        unsafe { raw::TEE_BigIntFMMContextSizeInU32(size) }
    }

    // Globalplatform define FMMContext1 here while OP-TEE does not update yet
    pub fn new(bits: u32, modulus: BigInt) -> Result<Self> {
        let size: usize = Self::size_in_u32(bits as usize);
        let mut tmp_vec: Vec<BigIntFMMContextUnit> = vec![0; size];
        unsafe {
            raw::TEE_BigIntInitFMMContext(tmp_vec.as_mut_ptr(), size, modulus.data_ptr())
        };
        Ok(Self(tmp_vec))
    }
}

pub struct BigIntFMM(Vec<BigIntFMMUnit>);

impl BigIntFMM {
    pub fn data_ptr(&self) -> *const u32 {
        self.0.as_ptr()
    }

    fn size_in_u32(size: usize) -> usize {
        unsafe { raw::TEE_BigIntFMMSizeInU32(size) }
    }

    pub fn new(bits: u32) -> Self {
        let size: usize = Self::size_in_u32(bits as usize);
        let mut tmp_vec: Vec<BigIntFMMUnit> = vec![0; size];
        unsafe { raw::TEE_BigIntInitFMM(tmp_vec.as_mut_ptr(), size) };
        Self(tmp_vec)
    }

    //Has to be initialized first
    pub fn convert_from_big_int(&mut self, src: &BigInt, n: &BigInt, context: BigIntFMMContext) {
        unsafe {
            raw::TEE_BigIntConvertToFMM(
                self.0.as_mut_ptr(),
                src.data_ptr(),
                n.data_ptr(),
                context.data_ptr(),
            )
        };
    }

    //Has to be initialized first
    pub fn compute_fmm(
        &mut self,
        op1: &BigIntFMM,
        op2: &BigIntFMM,
        n: &BigInt,
        context: BigIntFMMContext,
    ) {
        unsafe {
            raw::TEE_BigIntComputeFMM(
                self.0.as_mut_ptr(),
                op1.data_ptr(),
                op2.data_ptr(),
                n.data_ptr(),
                context.data_ptr(),
            )
        };
    }
}
//OP-TEE in version GP 1.1.1 does not implement function:
//TEE_BigIntSetBit
//TEE_BigIntAssign
//TEE_BigIntAbs
//TEE_BigIntExpMod
