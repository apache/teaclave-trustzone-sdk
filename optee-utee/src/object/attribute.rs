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

use core::marker;

use optee_utee_sys as raw;

/// A general attribute (buffer or value) that can be used to populate an object or to specify
/// operation parameters.
pub struct Attribute {
    raw: raw::TEE_Attribute,
}

impl Attribute {
    /// Return the raw struct `TEE_Attribute`.
    pub fn raw(&self) -> raw::TEE_Attribute {
        self.raw
    }
}

/// Convert the buffer attribute [AttributeMemref](crate::AttributeMemref) to
/// the general attribute.
impl<'attrref> From<AttributeMemref<'attrref>> for Attribute {
    fn from(attr: AttributeMemref) -> Self {
        Self { raw: attr.raw() }
    }
}

/// Convert the value attribute [AttributeValue](crate::AttributeValue) to
/// the general attribute.
impl From<AttributeValue> for Attribute {
    fn from(attr: AttributeValue) -> Self {
        Self { raw: attr.raw() }
    }
}

/// A buffer attribute.
#[derive(Clone, Copy)]
pub struct AttributeMemref<'attrref> {
    raw: raw::TEE_Attribute,
    _marker: marker::PhantomData<&'attrref mut [u8]>,
}

impl<'attrref> AttributeMemref<'attrref> {
    /// Return the raw struct TEE_Attribute.
    pub fn raw(&self) -> raw::TEE_Attribute {
        self.raw
    }

    fn new_ref() -> Self {
        let raw = raw::TEE_Attribute {
            attributeID: 0,
            content: raw::content {
                memref: raw::Memref {
                    buffer: core::ptr::null_mut(),
                    size: 0,
                },
            },
        };
        Self {
            raw,
            _marker: marker::PhantomData,
        }
    }

    /// Populate a single attribute with a reference to a buffer.
    ///
    /// # Parameters
    ///
    /// 1) `id`: The [AttributeId](crate::AttributeId) is an identifier of the
    ///    attribute to populate.
    /// 2) `buffer`: Input buffer that holds the content of the attribute.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{AttributeMemref, AttributeId};
    /// let mut attr = AttributeMemref::from_ref(AttributeId::SecretValue, &mut [0u8;1]);
    /// ```
    pub fn from_ref(id: AttributeId, buffer: &'attrref [u8]) -> Self {
        let mut res = AttributeMemref::new_ref();
        unsafe {
            raw::TEE_InitRefAttribute(
                &mut res.raw,
                id as u32,
                buffer.as_ptr() as *mut _,
                buffer.len(),
            );
        }
        res
    }
}

/// A value attribute.
pub struct AttributeValue {
    raw: raw::TEE_Attribute,
}

impl AttributeValue {
    /// Return the raw struct TEE_Attribute.
    pub fn raw(&self) -> raw::TEE_Attribute {
        self.raw
    }

    fn new_value() -> Self {
        let raw = raw::TEE_Attribute {
            attributeID: 0,
            content: raw::content {
                value: raw::Value { a: 0, b: 0 },
            },
        };
        Self { raw }
    }

    /// Populate a single attribute with two u32 values.
    ///
    /// # Parameters
    ///
    /// 1) `id`: The [AttributeId](crate::AttributeId) is an identifier of the
    ///    attribute to populate.
    /// 2) `a`, `b`: u32 values to assign to the members of the value attribute.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{AttributeValue, AttributeId};
    /// let mut attr = AttributeValue::from_value(AttributeId::SecretValue, 0, 0);
    /// ```
    pub fn from_value(id: AttributeId, a: u32, b: u32) -> Self {
        let mut res = AttributeValue::new_value();
        unsafe {
            raw::TEE_InitValueAttribute(&mut res.raw, id as u32, a, b);
        }
        res
    }
}

#[repr(u32)]
pub enum AttributeId {
    /// Used for all secret keys for symmetric ciphers, MACs, and HMACs
    SecretValue = 0xC0000000,
    /// RSA modulus: `n`
    RsaModulus = 0xD0000130,
    /// RSA public key exponent: `e`
    RsaPublicExponent = 0xD0000230,
    /// RSA private key exponent: `d`
    RsaPrivateExponent = 0xC0000330,
    /// RSA prime number: `p`
    RsaPrime1 = 0xC0000430,
    /// RSA prime number: `q`
    RsaPrime2 = 0xC0000530,
    /// RSA exponent: `dp`
    RsaExponent1 = 0xC0000630,
    /// RSA exponent: `dq`
    RsaExponent2 = 0xC0000730,
    /// RSA coefficient: `iq`
    RsaCoefficient = 0xC0000830,
    /// DSA prime number: `p`
    DsaPrime = 0xD0001031,
    /// DSA sub prime number: `q`
    DsaSubprime = 0xD0001131,
    /// DSA base: `g`
    DsaBase = 0xD0001231,
    /// DSA public value: `y`
    DsaPublicValue = 0xD0000131,
    /// DSA private value: `x`
    DsaPrivateValue = 0xC0000231,
    /// Diffie-Hellman prime number: `p`
    DhPrime = 0xD0001032,
    /// Diffie-Hellman subprime number: `q`
    DhSubprime = 0xD0001132,
    /// Diffie-Hellman base: `g`
    DhBase = 0xD0001232,
    /// Diffie-Hellman x bits: `l`
    DhXBits = 0xF0001332,
    /// Diffie-Hellman public value: `y`
    DhPublicValue = 0xD0000132,
    /// Diffie-Hellman public value: `x`
    DhPrivateValue = 0xC0000232,
    RsaOaepLabel = 0xD0000930,
    RsaOaepMgf1Hash = 0xD0000931,
    RsaPssSaltLength = 0xF0000A30,
    /// ECC public value: `x`
    EccPublicValueX = 0xD0000141,
    /// ECC public value: `y`
    EccPublicValueY = 0xD0000241,
    /// ECC private value: `d`
    EccPrivateValue = 0xC0000341,
    /// Ed25519 public value
    Ed25519PublicValue = 0xD0000743,
    /// Ed25519 private value
    Ed25519PrivateValue = 0xC0000843,
    /// X25519 public value
    X25519PublicValue = 0xD0000944,
    /// X25519 private value
    X25519PrivateValue = 0xC0000A44,
    /// ECC Curve algorithm
    EccCurve = 0xF0000441,
    BitProtected = (1 << 28),
    BitValue = (1 << 29),
}
