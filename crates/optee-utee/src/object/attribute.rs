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
            content: raw::TEE_Attribute__bindgen_ty_1 {
                ref_: raw::TEE_Attribute__bindgen_ty_1__bindgen_ty_1 {
                    buffer: core::ptr::null_mut(),
                    length: 0,
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
            content: raw::TEE_Attribute__bindgen_ty_1 {
                value: raw::TEE_Attribute__bindgen_ty_1__bindgen_ty_2 { a: 0, b: 0 },
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

#[derive(Clone, Copy, strum::Display)]
#[repr(u32)]
pub enum AttributeId {
    /// Used for all secret keys for symmetric ciphers, MACs, and HMACs
    SecretValue = raw::TEE_ATTR_SECRET_VALUE,
    /// RSA modulus: `n`
    RsaModulus = raw::TEE_ATTR_RSA_MODULUS,
    /// RSA public key exponent: `e`
    RsaPublicExponent = raw::TEE_ATTR_RSA_PUBLIC_EXPONENT,
    /// RSA private key exponent: `d`
    RsaPrivateExponent = raw::TEE_ATTR_RSA_PRIVATE_EXPONENT,
    /// RSA prime number: `p`
    RsaPrime1 = raw::TEE_ATTR_RSA_PRIME1,
    /// RSA prime number: `q`
    RsaPrime2 = raw::TEE_ATTR_RSA_PRIME2,
    /// RSA exponent: `dp`
    RsaExponent1 = raw::TEE_ATTR_RSA_EXPONENT1,
    /// RSA exponent: `dq`
    RsaExponent2 = raw::TEE_ATTR_RSA_EXPONENT2,
    /// RSA coefficient: `iq`
    RsaCoefficient = raw::TEE_ATTR_RSA_COEFFICIENT,
    /// DSA prime number: `p`
    DsaPrime = raw::TEE_ATTR_DSA_PRIME,
    /// DSA sub prime number: `q`
    DsaSubprime = raw::TEE_ATTR_DSA_SUBPRIME,
    /// DSA base: `g`
    DsaBase = raw::TEE_ATTR_DSA_BASE,
    /// DSA public value: `y`
    DsaPublicValue = raw::TEE_ATTR_DSA_PUBLIC_VALUE,
    /// DSA private value: `x`
    DsaPrivateValue = raw::TEE_ATTR_DSA_PRIVATE_VALUE,
    /// Diffie-Hellman prime number: `p`
    DhPrime = raw::TEE_ATTR_DH_PRIME,
    /// Diffie-Hellman subprime number: `q`
    DhSubprime = raw::TEE_ATTR_DH_SUBPRIME,
    /// Diffie-Hellman base: `g`
    DhBase = raw::TEE_ATTR_DH_BASE,
    /// Diffie-Hellman x bits: `l`
    DhXBits = raw::TEE_ATTR_DH_X_BITS,
    /// Diffie-Hellman public value: `y`
    DhPublicValue = raw::TEE_ATTR_DH_PUBLIC_VALUE,
    /// Diffie-Hellman public value: `x`
    DhPrivateValue = raw::TEE_ATTR_DH_PRIVATE_VALUE,
    /// PBKDF2 password, bytes of the password
    Pbkdf2Password = raw::TEE_ATTR_PBKDF2_PASSWORD,
    /// PBKDF2 salt, bytes of the salt
    Pbkdf2Salt = raw::TEE_ATTR_PBKDF2_SALT,
    /// PBKDF2 iteration count
    Pbkdf2IterationCount = raw::TEE_ATTR_PBKDF2_ITERATION_COUNT,
    /// PBKDF2 derived key material length in bytes
    Pbkdf2DkmLength = raw::TEE_ATTR_PBKDF2_DKM_LENGTH,
    RsaOaepLabel = raw::TEE_ATTR_RSA_OAEP_LABEL,
    RsaOaepMgf1Hash = raw::TEE_ATTR_RSA_OAEP_MGF_HASH,
    RsaPssSaltLength = raw::TEE_ATTR_RSA_PSS_SALT_LENGTH,
    /// ECC public value: `x`
    EccPublicValueX = raw::TEE_ATTR_ECC_PUBLIC_VALUE_X,
    /// ECC public value: `y`
    EccPublicValueY = raw::TEE_ATTR_ECC_PUBLIC_VALUE_Y,
    /// ECC private value: `d`
    EccPrivateValue = raw::TEE_ATTR_ECC_PRIVATE_VALUE,
    /// Ed25519 public value
    Ed25519PublicValue = raw::TEE_ATTR_ED25519_PUBLIC_VALUE,
    /// Ed25519 private value
    Ed25519PrivateValue = raw::TEE_ATTR_ED25519_PRIVATE_VALUE,
    /// X25519 public value
    X25519PublicValue = raw::TEE_ATTR_X25519_PUBLIC_VALUE,
    /// X25519 private value
    X25519PrivateValue = raw::TEE_ATTR_X25519_PRIVATE_VALUE,
    /// ECC Curve algorithm
    EccCurve = raw::TEE_ATTR_ECC_CURVE,
    BitProtected = raw::TEE_ATTR_BIT_PROTECTED,
    BitValue = raw::TEE_ATTR_BIT_VALUE,
}
