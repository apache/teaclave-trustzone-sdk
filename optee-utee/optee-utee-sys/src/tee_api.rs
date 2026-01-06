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

#[cfg_attr(feature = "mock", mockall::automock)]
pub mod api {
    use crate::libc_compat::{intmax_t, size_t};
    use crate::*;
    use core::ffi::*;

    extern "C" {
        // Property access functions

        pub fn TEE_GetPropertyAsString(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            valueBuffer: *mut c_char,
            valueBufferLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsBool(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            value: *mut bool,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsU32(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            value: *mut u32,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsU64(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            value: *mut u64,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsBinaryBlock(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            valueBuffer: *mut c_void,
            valueBufferLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsUUID(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            value: *mut TEE_UUID,
        ) -> TEE_Result;
        pub fn TEE_GetPropertyAsIdentity(
            propsetOrEnumerator: TEE_PropSetHandle,
            name: *const c_char,
            value: *mut TEE_Identity,
        ) -> TEE_Result;
        pub fn TEE_AllocatePropertyEnumerator(enumerator: *mut TEE_PropSetHandle) -> TEE_Result;
        pub fn TEE_FreePropertyEnumerator(enumerator: TEE_PropSetHandle);
        pub fn TEE_StartPropertyEnumerator(
            enumerator: TEE_PropSetHandle,
            propSet: TEE_PropSetHandle,
        );
        pub fn TEE_ResetPropertyEnumerator(enumerator: TEE_PropSetHandle);
        pub fn TEE_GetPropertyName(
            enumerator: TEE_PropSetHandle,
            nameBuffer: *mut c_void,
            nameBufferLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_GetNextProperty(enumerator: TEE_PropSetHandle) -> TEE_Result;

        // System API - Misc

        pub fn TEE_Panic(panicCode: TEE_Result);

        // System API - Internal Client API

        pub fn TEE_OpenTASession(
            destination: *const TEE_UUID,
            cancellationRequestTimeout: u32,
            paramTypes: u32,
            params: *mut TEE_Param,
            session: *mut TEE_TASessionHandle,
            returnOrigin: *mut u32,
        ) -> TEE_Result;
        pub fn TEE_CloseTASession(session: TEE_TASessionHandle);
        pub fn TEE_InvokeTACommand(
            session: TEE_TASessionHandle,
            cancellationRequestTimeout: u32,
            commandID: u32,
            paramTypes: u32,
            params: *mut TEE_Param,
            returnOrigin: *mut u32,
        ) -> TEE_Result;

        // System API - Cancellations

        pub fn TEE_GetCancellationFlag() -> bool;
        pub fn TEE_UnmaskCancellation() -> bool;
        pub fn TEE_MaskCancellation() -> bool;

        // System API - Memory Management

        pub fn TEE_CheckMemoryAccessRights(
            accessFlags: u32,
            buffer: *mut c_void,
            size: usize,
        ) -> TEE_Result;
        pub fn TEE_SetInstanceData(instanceData: *const c_void);
        pub fn TEE_GetInstanceData() -> *const c_void;
        pub fn TEE_Malloc(size: usize, hint: u32) -> *mut c_void;
        pub fn TEE_Realloc(buffer: *mut c_void, newSize: usize) -> *mut c_void;
        pub fn TEE_Free(buffer: *mut c_void);
        pub fn TEE_MemMove(dest: *mut c_void, src: *const c_void, size: usize);
        pub fn TEE_MemCompare(buffer1: *const c_void, buffer2: *const c_void, size: usize) -> i32;
        pub fn TEE_MemFill(buff: *mut c_void, x: u32, size: usize);

        // Data and Key Storage API  - Generic Object Functions

        pub fn TEE_GetObjectInfo(object: TEE_ObjectHandle, objectInfo: *mut TEE_ObjectInfo);
        pub fn TEE_GetObjectInfo1(
            object: TEE_ObjectHandle,
            objectInfo: *mut TEE_ObjectInfo,
        ) -> TEE_Result;
        pub fn TEE_RestrictObjectUsage(object: TEE_ObjectHandle, objectUsage: u32);
        pub fn TEE_RestrictObjectUsage1(object: TEE_ObjectHandle, objectUsage: u32) -> TEE_Result;
        pub fn TEE_GetObjectBufferAttribute(
            object: TEE_ObjectHandle,
            attributeID: u32,
            buffer: *mut c_void,
            size: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_GetObjectValueAttribute(
            object: TEE_ObjectHandle,
            attributeID: u32,
            a: *mut u32,
            b: *mut u32,
        ) -> TEE_Result;
        pub fn TEE_CloseObject(object: TEE_ObjectHandle);

        /* Data and Key Storage API  - Transient Object Functions */

        pub fn TEE_AllocateTransientObject(
            objectType: TEE_ObjectType,
            maxObjectSize: u32,
            object: *mut TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_FreeTransientObject(object: TEE_ObjectHandle) -> c_void;
        pub fn TEE_ResetTransientObject(object: TEE_ObjectHandle) -> c_void;
        pub fn TEE_PopulateTransientObject(
            object: TEE_ObjectHandle,
            attrs: *const TEE_Attribute,
            attrCount: u32,
        ) -> TEE_Result;
        pub fn TEE_InitRefAttribute(
            attr: *mut TEE_Attribute,
            attributeID: u32,
            buffer: *const c_void,
            length: usize,
        ) -> c_void;
        pub fn TEE_InitValueAttribute(
            attr: *mut TEE_Attribute,
            attributeID: u32,
            a: u32,
            b: u32,
        ) -> c_void;
        pub fn TEE_CopyObjectAttributes(
            destObject: TEE_ObjectHandle,
            srcObject: TEE_ObjectHandle,
        ) -> c_void;
        pub fn TEE_CopyObjectAttributes1(
            destObject: TEE_ObjectHandle,
            srcObject: TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_GenerateKey(
            object: TEE_ObjectHandle,
            keySize: u32,
            params: *const TEE_Attribute,
            paramCount: u32,
        ) -> TEE_Result;

        // Data and Key Storage API  - Persistent Object Functions

        pub fn TEE_OpenPersistentObject(
            storageID: u32,
            objectID: *const c_void,
            objectIDLen: usize,
            flags: u32,
            object: *mut TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_CreatePersistentObject(
            storageID: u32,
            objectID: *const c_void,
            objectIDLen: usize,
            flags: u32,
            attributes: TEE_ObjectHandle,
            initialData: *const c_void,
            initialDataLen: usize,
            object: *mut TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_CloseAndDeletePersistentObject(object: TEE_ObjectHandle);
        pub fn TEE_CloseAndDeletePersistentObject1(object: TEE_ObjectHandle) -> TEE_Result;
        pub fn TEE_RenamePersistentObject(
            object: TEE_ObjectHandle,
            newObjectID: *const c_void,
            newObjectIDLen: usize,
        ) -> TEE_Result;
        pub fn TEE_AllocatePersistentObjectEnumerator(
            objectEnumerator: *mut TEE_ObjectEnumHandle,
        ) -> TEE_Result;
        pub fn TEE_FreePersistentObjectEnumerator(objectEnumerator: TEE_ObjectEnumHandle);
        pub fn TEE_ResetPersistentObjectEnumerator(objectEnumerator: TEE_ObjectEnumHandle);
        pub fn TEE_StartPersistentObjectEnumerator(
            objectEnumerator: TEE_ObjectEnumHandle,
            storageID: u32,
        ) -> TEE_Result;
        pub fn TEE_GetNextPersistentObject(
            objectEnumerator: TEE_ObjectEnumHandle,
            objectInfo: *mut TEE_ObjectInfo,
            objectID: *mut c_void,
            objectIDLen: *mut usize,
        ) -> TEE_Result;

        // Data and Key Storage API  - Data Stream Access Functions

        pub fn TEE_ReadObjectData(
            object: TEE_ObjectHandle,
            buffer: *mut c_void,
            size: usize,
            count: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_WriteObjectData(
            object: TEE_ObjectHandle,
            buffer: *const c_void,
            size: usize,
        ) -> TEE_Result;
        pub fn TEE_TruncateObjectData(object: TEE_ObjectHandle, size: usize) -> TEE_Result;
        pub fn TEE_SeekObjectData(
            object: TEE_ObjectHandle,
            offset: intmax_t,
            whence: TEE_Whence,
        ) -> TEE_Result;

        // Cryptographic Operations API - Generic Operation Functions

        pub fn TEE_AllocateOperation(
            operation: *mut TEE_OperationHandle,
            algorithm: u32,
            mode: u32,
            maxKeySize: u32,
        ) -> TEE_Result;
        pub fn TEE_FreeOperation(operation: TEE_OperationHandle) -> c_void;
        pub fn TEE_GetOperationInfo(
            operation: TEE_OperationHandle,
            operationInfo: *mut TEE_OperationInfo,
        ) -> c_void;
        pub fn TEE_GetOperationInfoMultiple(
            operation: TEE_OperationHandle,
            operationInfoMultiple: *mut TEE_OperationInfoMultiple,
            operationSize: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_ResetOperation(operation: TEE_OperationHandle) -> c_void;
        pub fn TEE_SetOperationKey(
            operation: TEE_OperationHandle,
            key: TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_SetOperationKey2(
            operation: TEE_OperationHandle,
            key1: TEE_ObjectHandle,
            key2: TEE_ObjectHandle,
        ) -> TEE_Result;
        pub fn TEE_CopyOperation(
            dstOperation: TEE_OperationHandle,
            srcOperation: TEE_OperationHandle,
        ) -> c_void;
        pub fn TEE_IsAlgorithmSupported(algId: u32, element: u32) -> TEE_Result;

        // Cryptographic Operations API - Message Digest Functions

        pub fn TEE_DigestUpdate(
            operation: TEE_OperationHandle,
            chunk: *const c_void,
            chunkSize: usize,
        ) -> c_void;
        pub fn TEE_DigestDoFinal(
            operation: TEE_OperationHandle,
            chunk: *const c_void,
            chunkLen: usize,
            hash: *mut c_void,
            hashLen: *mut usize,
        ) -> TEE_Result;

        // Cryptographic Operations API - Symmetric Cipher Functions

        pub fn TEE_CipherInit(
            operation: TEE_OperationHandle,
            IV: *const c_void,
            IVLen: usize,
        ) -> c_void;
        pub fn TEE_CipherUpdate(
            operation: TEE_OperationHandle,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_CipherDoFinal(
            operation: TEE_OperationHandle,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
        ) -> TEE_Result;

        // Cryptographic Operations API - MAC Functions

        pub fn TEE_MACInit(
            operation: TEE_OperationHandle,
            IV: *const c_void,
            IVLen: usize,
        ) -> c_void;
        pub fn TEE_MACUpdate(
            operation: TEE_OperationHandle,
            chunk: *const c_void,
            chunkSize: usize,
        ) -> c_void;
        pub fn TEE_MACComputeFinal(
            operation: TEE_OperationHandle,
            message: *const c_void,
            messageLen: usize,
            mac: *mut c_void,
            macLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_MACCompareFinal(
            operation: TEE_OperationHandle,
            message: *const c_void,
            messageLen: usize,
            mac: *const c_void,
            macLen: usize,
        ) -> TEE_Result;

        // Cryptographic Operations API - Authenticated Encryption Functions

        pub fn TEE_AEInit(
            operation: TEE_OperationHandle,
            nonce: *const c_void,
            nonceLen: usize,
            tagLen: u32,
            AADLen: usize,
            payloadLen: usize,
        ) -> TEE_Result;
        pub fn TEE_AEUpdateAAD(
            operation: TEE_OperationHandle,
            AADdata: *const c_void,
            AADdataLen: usize,
        ) -> c_void;
        pub fn TEE_AEUpdate(
            operation: TEE_OperationHandle,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_AEEncryptFinal(
            operation: TEE_OperationHandle,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
            tag: *mut c_void,
            tagLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_AEDecryptFinal(
            operation: TEE_OperationHandle,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
            tag: *mut c_void,
            tagLen: usize,
        ) -> TEE_Result;

        // Cryptographic Operations API - Asymmetric Functions

        pub fn TEE_AsymmetricEncrypt(
            operation: TEE_OperationHandle,
            params: *const TEE_Attribute,
            paramCount: u32,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_AsymmetricDecrypt(
            operation: TEE_OperationHandle,
            params: *const TEE_Attribute,
            paramCount: u32,
            srcData: *const c_void,
            srcLen: usize,
            destData: *mut c_void,
            destLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_AsymmetricSignDigest(
            operation: TEE_OperationHandle,
            params: *const TEE_Attribute,
            paramCount: u32,
            digest: *const c_void,
            digestLen: usize,
            signature: *mut c_void,
            signatureLen: *mut usize,
        ) -> TEE_Result;
        pub fn TEE_AsymmetricVerifyDigest(
            operation: TEE_OperationHandle,
            params: *const TEE_Attribute,
            paramCount: u32,
            digest: *const c_void,
            digestLen: usize,
            signature: *const c_void,
            signatureLen: usize,
        ) -> TEE_Result;

        // Cryptographic Operations API - Key Derivation Functions

        pub fn TEE_DeriveKey(
            operation: TEE_OperationHandle,
            params: *const TEE_Attribute,
            paramCount: u32,
            derivedKey: TEE_ObjectHandle,
        ) -> c_void;

        // Cryptographic Operations API - Random Number Generation Functions

        pub fn TEE_GenerateRandom(randomBuffer: *mut c_void, randomBufferLen: usize) -> c_void;

        // Date & Time API

        pub fn TEE_GetSystemTime(time: *mut TEE_Time) -> c_void;
        pub fn TEE_Wait(timeout: u32) -> TEE_Result;
        pub fn TEE_GetTAPersistentTime(time: *mut TEE_Time) -> TEE_Result;
        pub fn TEE_SetTAPersistentTime(time: *const TEE_Time) -> TEE_Result;
        pub fn TEE_GetREETime(time: *mut TEE_Time) -> c_void;

        // TEE Arithmetical API - Memory allocation and size of objects

        pub fn TEE_BigIntFMMSizeInU32(modulusSizeInBits: usize) -> usize;
        pub fn TEE_BigIntFMMContextSizeInU32(modulusSizeInBits: usize) -> usize;

        // TEE Arithmetical API - Initialization functions

        pub fn TEE_BigIntInit(bigInt: *mut TEE_BigInt, len: usize) -> c_void;
        pub fn TEE_BigIntInitFMMContext(
            context: *mut TEE_BigIntFMMContext,
            len: usize,
            modulus: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntInitFMM(bigIntFMM: *mut TEE_BigIntFMM, len: usize) -> c_void;

        // TEE Arithmetical API - Converter functions

        pub fn TEE_BigIntConvertFromOctetString(
            dest: *mut TEE_BigInt,
            buffer: *const u8,
            bufferLen: usize,
            sign: i32,
        ) -> TEE_Result;
        pub fn TEE_BigIntConvertToOctetString(
            buffer: *mut u8,
            bufferLen: *mut usize,
            bigInt: *const TEE_BigInt,
        ) -> TEE_Result;
        pub fn TEE_BigIntConvertFromS32(dest: *mut TEE_BigInt, shortVal: i32) -> c_void;
        pub fn TEE_BigIntConvertToS32(dest: *mut i32, src: *const TEE_BigInt) -> TEE_Result;

        // TEE Arithmetical API - Logical operations

        pub fn TEE_BigIntCmp(op1: *const TEE_BigInt, op2: *const TEE_BigInt) -> i32;
        pub fn TEE_BigIntCmpS32(op: *const TEE_BigInt, shortVal: i32) -> i32;
        pub fn TEE_BigIntShiftRight(
            dest: *mut TEE_BigInt,
            op: *const TEE_BigInt,
            bits: size_t,
        ) -> c_void;
        pub fn TEE_BigIntGetBit(src: *const TEE_BigInt, bitIndex: u32) -> bool;
        pub fn TEE_BigIntGetBitCount(src: *const TEE_BigInt) -> u32;
        pub fn TEE_BigIntSetBit(src: *mut TEE_BigInt, bitIndex: u32, value: bool) -> TEE_Result;
        pub fn TEE_BigIntAssign(dest: *mut TEE_BigInt, src: *const TEE_BigInt) -> TEE_Result;
        pub fn TEE_BigIntAbs(dest: *mut TEE_BigInt, src: *const TEE_BigInt) -> TEE_Result;
        pub fn TEE_BigIntAdd(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntSub(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntNeg(dest: *mut TEE_BigInt, op: *const TEE_BigInt) -> c_void;
        pub fn TEE_BigIntMul(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntSquare(dest: *mut TEE_BigInt, op: *const TEE_BigInt) -> c_void;
        pub fn TEE_BigIntDiv(
            dest_q: *mut TEE_BigInt,
            dest_r: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
        ) -> c_void;

        // TEE Arithmetical API - Modular arithmetic operations

        pub fn TEE_BigIntMod(
            dest: *mut TEE_BigInt,
            op: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntAddMod(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntSubMod(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntMulMod(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntSquareMod(
            dest: *mut TEE_BigInt,
            op: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntInvMod(
            dest: *mut TEE_BigInt,
            op: *const TEE_BigInt,
            n: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntExpMod(
            dest: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
            n: *const TEE_BigInt,
            context: *const TEE_BigIntFMMContext,
        ) -> TEE_Result;

        // TEE Arithmetical API - Other arithmetic operations

        pub fn TEE_BigIntRelativePrime(op1: *const TEE_BigInt, op2: *const TEE_BigInt) -> bool;
        pub fn TEE_BigIntComputeExtendedGcd(
            gcd: *mut TEE_BigInt,
            u: *mut TEE_BigInt,
            v: *mut TEE_BigInt,
            op1: *const TEE_BigInt,
            op2: *const TEE_BigInt,
        ) -> c_void;
        pub fn TEE_BigIntIsProbablePrime(op: *const TEE_BigInt, confidenceLevel: u32) -> i32;

        // TEE Arithmetical API - Fast modular multiplication operations

        pub fn TEE_BigIntConvertToFMM(
            dest: *mut TEE_BigIntFMM,
            src: *const TEE_BigInt,
            n: *const TEE_BigInt,
            context: *const TEE_BigIntFMMContext,
        ) -> c_void;
        pub fn TEE_BigIntConvertFromFMM(
            dest: *mut TEE_BigInt,
            src: *const TEE_BigIntFMM,
            n: *const TEE_BigInt,
            context: *const TEE_BigIntFMMContext,
        ) -> c_void;
        pub fn TEE_BigIntFMMConvertToBigInt(
            dest: *mut TEE_BigInt,
            src: *const TEE_BigIntFMM,
            n: *const TEE_BigInt,
            context: *const TEE_BigIntFMMContext,
        ) -> c_void;
        pub fn TEE_BigIntComputeFMM(
            dest: *mut TEE_BigIntFMM,
            op1: *const TEE_BigIntFMM,
            op2: *const TEE_BigIntFMM,
            n: *const TEE_BigInt,
            context: *const TEE_BigIntFMMContext,
        ) -> c_void;
    }
}
