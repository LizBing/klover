/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::{ffi::c_char, os::raw::c_void};

// Integral types
pub type jbyte  = i8;
pub type jshort = i16;
pub type jint   = i32;
pub type jlong  = i64;
pub type jchar  = u16;

// Floating-point types
pub type jfloat  = f32;
pub type jdouble = f64;

// Boolean type
pub type jboolean = u8;

pub type jsize    = jint;

pub struct _jobject;
pub type jobject = *mut _jobject; 
pub type jclass         = jobject;
pub type jthrowable     = jobject;
pub type jstring        = jobject;
pub type jarray         = jobject;
pub type jbooleanArray  = jarray;
pub type jbyteArray     = jarray;
pub type jcharArray     = jarray;
pub type jshortArray    = jarray;
pub type jintArray      = jarray;
pub type jlongArray     = jarray;
pub type jfloatArray    = jarray;
pub type jdoubleArray   = jarray;
pub type jobjectArray   = jarray;

pub type jweak = jobject;

#[repr(C)]
pub union jvalue {
    z: jboolean,
    b: jbyte,
    c: jchar,
    s: jshort,
    i: jint,
    j: jlong,
    f: jfloat,
    d: jdouble,
    l: jobject
}

pub struct _jfieldID;
pub type jfieldID = *mut _jfieldID;

pub struct _jmethodID;
pub type jmethodID = *mut _jmethodID;

#[repr(C)]
pub enum jobjectRefType {
    JNIInvalidRefType    = 0,
    JNILocalRefType      = 1,
    JNIGlobalRefType     = 2,
    JNIWeakGlobalRefType = 3
}

// jboolean constants
pub const JNI_FALSE: jboolean = 0;
pub const JNI_TRUE:  jboolean = 1;

// possible return values for JNI functions.
pub const JNI_OK:           jint = 0;
pub const JNI_ERR:          jint = -1;
pub const JNI_EDETACHED:    jint = -2;
pub const JNI_EVERSION:     jint = -3;
pub const JNI_ENOMEM:       jint = -4;
pub const JNI_EEXIST:       jint = -5;
pub const JNI_EINVAL:       jint = -6;

// used in ReleaseScalarArrayElements
pub const JNI_COMMIT: jint = 1;
pub const JNI_ABORT: jint = 2;

#[repr(C)]
pub struct JNINativeMethod {
    pub name: *mut c_char,
    pub signature: *mut c_char,
    pub fnPtr: *mut c_void,
}

pub type JNIEnv = *mut JNINativeInterface;

#[repr(C)]
pub struct JNINativeInterface {
    pub reserved0: *mut c_void,
    pub reserved1: *mut c_void,
    pub reserved2: *mut c_void,
    pub reserved3: *mut c_void,

    pub GetVersion: extern "C" fn(*mut JNIEnv) -> jint,

    pub DefineClass: extern "C" fn(
        *mut JNIEnv,
        *const c_char,
        jobject,
        *const jbyte,
        jsize,
    ) -> jclass,

    pub FindClass: extern "C" fn(*mut JNIEnv, *const c_char) -> jclass,

    pub FromReflectedMethod: extern "C" fn(*mut JNIEnv, jobject) -> jmethodID,
    pub FromReflectedField: extern "C" fn(*mut JNIEnv, jobject) -> jfieldID,

    pub ToReflectedMethod: extern "C" fn(
        *mut JNIEnv,
        jclass,
        jmethodID,
        jboolean,
    ) -> jobject,

    pub GetSuperclass: extern "C" fn(*mut JNIEnv, jclass) -> jclass,
    pub IsAssignableFrom: extern "C" fn(*mut JNIEnv, jclass, jclass) -> jboolean,

    pub ToReflectedField: extern "C" fn(
        *mut JNIEnv,
        jclass,
        jfieldID,
        jboolean,
    ) -> jobject,

    pub Throw: extern "C" fn(*mut JNIEnv, jthrowable) -> jint,
    pub ThrowNew: extern "C" fn(*mut JNIEnv, jclass, *const c_char) -> jint,
    pub ExceptionOccurred: extern "C" fn(*mut JNIEnv) -> jthrowable,
    pub ExceptionDescribe: extern "C" fn(*mut JNIEnv),
    pub ExceptionClear: extern "C" fn(*mut JNIEnv),
    pub FatalError: extern "C" fn(*mut JNIEnv, *const c_char),

    pub PushLocalFrame: extern "C" fn(*mut JNIEnv, jint) -> jint,
    pub PopLocalFrame: extern "C" fn(*mut JNIEnv, jobject) -> jobject,

    pub NewGlobalRef: extern "C" fn(*mut JNIEnv, jobject) -> jobject,
    pub DeleteGlobalRef: extern "C" fn(*mut JNIEnv, jobject),
    pub DeleteLocalRef: extern "C" fn(*mut JNIEnv, jobject),
    pub IsSameObject: extern "C" fn(*mut JNIEnv, jobject, jobject) -> jboolean,
    pub NewLocalRef: extern "C" fn(*mut JNIEnv, jobject) -> jobject,
    pub EnsureLocalCapacity: extern "C" fn(*mut JNIEnv, jint) -> jint,

    pub AllocObject: extern "C" fn(*mut JNIEnv, jclass) -> jobject,

    pub NewObject: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jobject,
    pub NewObjectV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jobject,
    pub NewObjectA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jobject,

    pub GetObjectClass: extern "C" fn(*mut JNIEnv, jobject) -> jclass,
    pub IsInstanceOf: extern "C" fn(*mut JNIEnv, jobject, jclass) -> jboolean,

    pub GetMethodID: extern "C" fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jmethodID,

    pub CallObjectMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jobject,
    pub CallObjectMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jobject,
    pub CallObjectMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jobject,

    pub CallBooleanMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jboolean,
    pub CallBooleanMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jboolean,
    pub CallBooleanMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jboolean,

    pub CallByteMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jbyte,
    pub CallByteMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jbyte,
    pub CallByteMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jbyte,

    pub CallCharMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jchar,
    pub CallCharMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jchar,
    pub CallCharMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jchar,

    pub CallShortMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jshort,
    pub CallShortMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jshort,
    pub CallShortMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jshort,

    pub CallIntMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jint,
    pub CallIntMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jint,
    pub CallIntMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jint,

    pub CallLongMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jlong,
    pub CallLongMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jlong,
    pub CallLongMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jlong,

    pub CallFloatMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jfloat,
    pub CallFloatMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jfloat,
    pub CallFloatMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jfloat,

    pub CallDoubleMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jdouble,
    pub CallDoubleMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jdouble,
    pub CallDoubleMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jdouble,

    pub CallVoidMethod: extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...),
    pub CallVoidMethodV: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *mut c_char),
    pub CallVoidMethodA: extern "C" fn(*mut JNIEnv, jobject, jmethodID, *const jvalue),

    pub CallNonvirtualObjectMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jobject,
    pub CallNonvirtualObjectMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jobject,
    pub CallNonvirtualObjectMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jobject,

    pub CallNonvirtualBooleanMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jboolean,
    pub CallNonvirtualBooleanMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jboolean,
    pub CallNonvirtualBooleanMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jboolean,

    pub CallNonvirtualByteMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jbyte,
    pub CallNonvirtualByteMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jbyte,
    pub CallNonvirtualByteMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jbyte,

    pub CallNonvirtualCharMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jchar,
    pub CallNonvirtualCharMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jchar,
    pub CallNonvirtualCharMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jchar,

    pub CallNonvirtualShortMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jshort,
    pub CallNonvirtualShortMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jshort,
    pub CallNonvirtualShortMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jshort,

    pub CallNonvirtualIntMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jint,
    pub CallNonvirtualIntMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jint,
    pub CallNonvirtualIntMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jint,

    pub CallNonvirtualLongMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jlong,
    pub CallNonvirtualLongMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jlong,
    pub CallNonvirtualLongMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jlong,

    pub CallNonvirtualFloatMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jfloat,
    pub CallNonvirtualFloatMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jfloat,
    pub CallNonvirtualFloatMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jfloat,

    pub CallNonvirtualDoubleMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jdouble,
    pub CallNonvirtualDoubleMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jdouble,
    pub CallNonvirtualDoubleMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jdouble,

    pub CallNonvirtualVoidMethod: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...),
    pub CallNonvirtualVoidMethodV: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char),
    pub CallNonvirtualVoidMethodA: extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue),

    pub GetFieldID: extern "C" fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jfieldID,

    pub GetObjectField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jobject,
    pub GetBooleanField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jboolean,
    pub GetByteField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jbyte,
    pub GetCharField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jchar,
    pub GetShortField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jshort,
    pub GetIntField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jint,
    pub GetLongField: extern "C" fn(*mut JNIEnv, jobject, jfieldID)
    -> jlong,
    pub GetFloatField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jfloat,
    pub GetDoubleField: extern "C" fn(*mut JNIEnv, jobject, jfieldID) -> jdouble,

    pub SetObjectField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jobject),
    pub SetBooleanField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jboolean),
    pub SetByteField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jbyte),
    pub SetCharField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jchar),
    pub SetShortField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jshort),
    pub SetIntField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jint),
    pub SetLongField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jlong),
    pub SetFloatField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jfloat),
    pub SetDoubleField: extern "C" fn(*mut JNIEnv, jobject, jfieldID, jdouble),

    pub GetStaticMethodID: extern "C" fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jmethodID,

    pub CallStaticObjectMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jobject,
    pub CallStaticObjectMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jobject,
    pub CallStaticObjectMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jobject,

    pub CallStaticBooleanMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jboolean,
    pub CallStaticBooleanMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jboolean,
    pub CallStaticBooleanMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jboolean,

    pub CallStaticByteMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jbyte,
    pub CallStaticByteMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jbyte,
    pub CallStaticByteMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jbyte,

    pub CallStaticCharMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jchar,
    pub CallStaticCharMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jchar,
    pub CallStaticCharMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jchar,

    pub CallStaticShortMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jshort,
    pub CallStaticShortMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jshort,
    pub CallStaticShortMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jshort,

    pub CallStaticIntMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jint,
    pub CallStaticIntMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jint,
    pub CallStaticIntMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jint,

    pub CallStaticLongMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jlong,
    pub CallStaticLongMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jlong,
    pub CallStaticLongMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jlong,

    pub CallStaticFloatMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jfloat,
    pub CallStaticFloatMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jfloat,
    pub CallStaticFloatMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jfloat,

    pub CallStaticDoubleMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jdouble,
    pub CallStaticDoubleMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jdouble,
    pub CallStaticDoubleMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jdouble,

    pub CallStaticVoidMethod: extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...),
    pub CallStaticVoidMethodV: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *mut c_char),
    pub CallStaticVoidMethodA: extern "C" fn(*mut JNIEnv, jclass, jmethodID, *const jvalue),

    pub GetStaticFieldID: extern "C" fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jfieldID,
    pub GetStaticObjectField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jobject,
    pub GetStaticBooleanField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jboolean,
    pub GetStaticByteField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jbyte,
    pub GetStaticCharField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jchar,
    pub GetStaticShortField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jshort,
    pub GetStaticIntField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jint,
    pub GetStaticLongField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jlong,
    pub GetStaticFloatField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jfloat,
    pub GetStaticDoubleField: extern "C" fn(*mut JNIEnv, jclass, jfieldID) -> jdouble,

    pub SetStaticObjectField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jobject),
    pub SetStaticBooleanField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jboolean),
    pub SetStaticByteField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jbyte),
    pub SetStaticCharField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jchar),
    pub SetStaticShortField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jshort),
    pub SetStaticIntField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jint),
    pub SetStaticLongField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jlong),
    pub SetStaticFloatField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jfloat),
    pub SetStaticDoubleField: extern "C" fn(*mut JNIEnv, jclass, jfieldID, jdouble),

    pub NewString: extern "C" fn(*mut JNIEnv, *const jchar, jsize) -> jstring,
    pub GetStringLength: extern "C" fn(*mut JNIEnv, jstring) -> jsize,
    pub GetStringChars: extern "C" fn(*mut JNIEnv, jstring, *mut jboolean) -> *const jchar,
    pub ReleaseStringChars: extern "C" fn(*mut JNIEnv, jstring, *const jchar),

    pub NewStringUTF: extern "C" fn(*mut JNIEnv, *const c_char) -> jstring,
    pub GetStringUTFLength: extern "C" fn(*mut JNIEnv, jstring) -> jsize,
    pub GetStringUTFChars: extern "C" fn(*mut JNIEnv, jstring, *mut jboolean) -> *const c_char,
    pub ReleaseStringUTFChars: extern "C" fn(*mut JNIEnv, jstring, *const c_char),

    pub GetArrayLength: extern "C" fn(*mut JNIEnv, jarray) -> jsize,

    pub NewObjectArray: extern "C" fn(*mut JNIEnv, jsize, jclass, jobject) -> jobjectArray,
    pub GetObjectArrayElement: extern "C" fn(*mut JNIEnv, jobjectArray, jsize) -> jobject,
    pub SetObjectArrayElement: extern "C" fn(*mut JNIEnv, jobjectArray, jsize, jobject),

    pub NewBooleanArray: extern "C" fn(*mut JNIEnv, jsize) -> jbooleanArray,
    pub NewByteArray: extern "C" fn(*mut JNIEnv, jsize) -> jbyteArray,
    pub NewCharArray: extern "C" fn(*mut JNIEnv, jsize) -> jcharArray,
    pub NewShortArray: extern "C" fn(*mut JNIEnv, jsize) -> jshortArray,
    pub NewIntArray: extern "C" fn(*mut JNIEnv, jsize) -> jintArray,
    pub NewLongArray: extern "C" fn(*mut JNIEnv, jsize) -> jlongArray,
    pub NewFloatArray: extern "C" fn(*mut JNIEnv, jsize) -> jfloatArray,
    pub NewDoubleArray: extern "C" fn(*mut JNIEnv, jsize) -> jdoubleArray,

    pub GetBooleanArrayElements: extern "C" fn(*mut JNIEnv, jbooleanArray, *mut jboolean) -> *mut jboolean,
    pub GetByteArrayElements: extern "C" fn(*mut JNIEnv, jbyteArray, *mut jboolean) -> *mut jbyte,
    pub GetCharArrayElements: extern "C" fn(*mut JNIEnv, jcharArray, *mut jboolean) -> *mut jchar,
    pub GetShortArrayElements: extern "C" fn(*mut JNIEnv, jshortArray, *mut jboolean) -> *mut jshort,
    pub GetIntArrayElements: extern "C" fn(*mut JNIEnv, jintArray, *mut jboolean) -> *mut jint,
    pub GetLongArrayElements: extern "C" fn(*mut JNIEnv, jlongArray, *mut jboolean) -> *mut jlong,
    pub GetFloatArrayElements: extern "C" fn(*mut JNIEnv, jfloatArray, *mut jboolean) -> *mut jfloat,
    pub GetDoubleArrayElements: extern "C" fn(*mut JNIEnv, jdoubleArray, *mut jboolean) -> *mut jdouble,

    pub ReleaseBooleanArrayElements: extern "C" fn(*mut JNIEnv, jbooleanArray, *mut jboolean, jint),
    pub ReleaseByteArrayElements: extern "C" fn(*mut JNIEnv, jbyteArray, *mut jbyte, jint),
    pub ReleaseCharArrayElements: extern "C" fn(*mut JNIEnv, jcharArray, *mut jchar, jint),
    pub ReleaseShortArrayElements: extern "C" fn(*mut JNIEnv, jshortArray, *mut jshort, jint),
    pub ReleaseIntArrayElements: extern "C" fn(*mut JNIEnv, jintArray, *mut jint, jint),
    pub ReleaseLongArrayElements: extern "C" fn(*mut JNIEnv, jlongArray, *mut jlong, jint),
    pub ReleaseFloatArrayElements: extern "C" fn(*mut JNIEnv, jfloatArray, *mut jfloat, jint),
    pub ReleaseDoubleArrayElements: extern "C" fn(*mut JNIEnv, jdoubleArray, *mut jdouble, jint),

    pub GetBooleanArrayRegion: extern "C" fn(*mut JNIEnv, jbooleanArray, jsize, jsize, *mut jboolean),
    pub GetByteArrayRegion: extern "C" fn(*mut JNIEnv, jbyteArray, jsize, jsize, *mut jbyte),
    pub GetCharArrayRegion: extern "C" fn(*mut JNIEnv, jcharArray, jsize, jsize, *mut jchar),
    pub GetShortArrayRegion: extern "C" fn(*mut JNIEnv, jshortArray, jsize, jsize, *mut jshort),
    pub GetIntArrayRegion: extern "C" fn(*mut JNIEnv, jintArray, jsize, jsize, *mut jint),
    pub GetLongArrayRegion: extern "C" fn(*mut JNIEnv, jlongArray, jsize, jsize, *mut jlong),
    pub GetFloatArrayRegion: extern "C" fn(*mut JNIEnv, jfloatArray, jsize, jsize, *mut jfloat),
    pub GetDoubleArrayRegion: extern "C" fn(*mut JNIEnv, jdoubleArray, jsize, jsize, *mut jdouble),

    pub SetBooleanArrayRegion: extern "C" fn(*mut JNIEnv, jbooleanArray, jsize, jsize, *const jboolean),
    pub SetByteArrayRegion: extern "C" fn(*mut JNIEnv, jbyteArray, jsize, jsize, *const jbyte),
    pub SetCharArrayRegion: extern "C" fn(*mut JNIEnv, jcharArray, jsize, jsize, *const jchar),
    pub SetShortArrayRegion: extern "C" fn(*mut JNIEnv, jshortArray, jsize, jsize, *const jshort),
    pub SetIntArrayRegion: extern "C" fn(*mut JNIEnv, jintArray, jsize, jsize, *const jint),
    pub SetLongArrayRegion: extern "C" fn(*mut JNIEnv, jlongArray, jsize, jsize, *const jlong),
    pub SetFloatArrayRegion: extern "C" fn(*mut JNIEnv, jfloatArray, jsize, jsize, *const jfloat),
    pub SetDoubleArrayRegion: extern "C" fn(*mut JNIEnv, jdoubleArray, jsize, jsize, *const jdouble),

    pub RegisterNatives: extern "C" fn(*mut JNIEnv, jclass, *const JNINativeMethod, jint) -> jint,
    pub UnregisterNatives: extern "C" fn(*mut JNIEnv, jclass) -> jint,

    pub MonitorEnter: extern "C" fn(*mut JNIEnv, jobject) -> jint,
    pub MonitorExit: extern "C" fn(*mut JNIEnv, jobject) -> jint,

    pub GetJavaVM: extern "C" fn(*mut JNIEnv, *mut *mut c_void) -> jint,

    pub GetStringRegion: extern "C" fn(*mut JNIEnv, jstring, jsize, jsize, *mut jchar),
    pub GetStringUTFRegion: extern "C" fn(*mut JNIEnv, jstring, jsize, jsize, *mut c_char),

    pub GetPrimitiveArrayCritical: extern "C" fn(*mut JNIEnv, jarray, *mut jboolean) -> *mut c_void,
    pub ReleasePrimitiveArrayCritical: extern "C" fn(*mut JNIEnv, jarray, *mut c_void, jint),

    pub GetStringCritical: extern "C" fn(*mut JNIEnv, jstring, *mut jboolean) -> *mut c_char,
    pub ReleaseStringCritical: extern "C" fn(*mut JNIEnv, jstring, *const c_char),

    pub NewWeakGlobalRef: extern "C" fn(*mut JNIEnv, jobject) -> jweak,
    pub DeleteWeakGlobalRef: extern "C" fn(*mut JNIEnv, jweak),

    pub ExceptionCheck: extern "C" fn(*mut JNIEnv) -> jboolean,

    pub NewDirectByteBuffer: extern "C" fn(*mut JNIEnv, *mut c_void, jlong) -> jobject,
    pub GetDirectBufferAddress: extern "C" fn(*mut JNIEnv, jobject) -> *mut c_void,
    pub GetDirectBufferCapacity: extern "C" fn(*mut JNIEnv, jobject) -> jlong,

    pub GetObjectRefType: extern "C" fn(*mut JNIEnv, jobject) -> jobjectRefType,

    pub GetModule: extern "C" fn(*mut JNIEnv, jobject) -> jobject,

    pub IsVirtualThread: extern "C" fn(*mut JNIEnv, jobject) -> jboolean,
}

#[repr(C)]
pub struct JavaVMOption {
    optionString: *mut c_char,
    extraInfo: *mut c_void,
}

#[repr(C)]
pub struct JavaVMInitArgs {
    version: jint,
    nOptions: jint,
    options: *mut JavaVMOption,
    ignoreUnrecognized: jboolean,
}

#[repr(C)]
pub struct JavaVMAttachArgs {
    version: jint,
    name: *mut c_char,
    group: jobject,
}

#[repr(C)]
pub struct JNIInvokeInterface {
    pub reserved0: *mut c_void,
    pub reserved1: *mut c_void,
    pub reserved2: *mut c_void,

    pub DestroyJavaVM: extern "C" fn(*mut JavaVM) -> jint,
    pub AttachCurrentThread: extern "C" fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
    pub DetachCurrentThread: extern "C" fn(*mut JavaVM) -> jint,
    pub GetEnv: extern "C" fn(*mut JavaVM, *mut *mut c_void, jint) -> jint,
    pub AttachCurrentThreadAsDaemon: extern "C" fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
}

pub type JavaVM = *mut JNIInvokeInterface;

pub const JNI_VERSION: jint = 0x00150000;
