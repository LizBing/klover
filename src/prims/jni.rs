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

    pub GetVersion: fn(*mut JNIEnv) -> jint,

    pub DefineClass: fn(
        *mut JNIEnv,
        *const c_char,
        jobject,
        *const jbyte,
        jsize,
    ) -> jclass,

    pub FindClass: fn(*mut JNIEnv, *const c_char) -> jclass,

    pub FromReflectedMethod: fn(*mut JNIEnv, jobject) -> jmethodID,
    pub FromReflectedField: fn(*mut JNIEnv, jobject) -> jfieldID,

    pub ToReflectedMethod: fn(
        *mut JNIEnv,
        jclass,
        jmethodID,
        jboolean,
    ) -> jobject,

    pub GetSuperclass: fn(*mut JNIEnv, jclass) -> jclass,
    pub IsAssignableFrom: fn(*mut JNIEnv, jclass, jclass) -> jboolean,

    pub ToReflectedField: fn(
        *mut JNIEnv,
        jclass,
        jfieldID,
        jboolean,
    ) -> jobject,

    pub Throw: fn(*mut JNIEnv, jthrowable) -> jint,
    pub ThrowNew: fn(*mut JNIEnv, jclass, *const c_char) -> jint,
    pub ExceptionOccurred: fn(*mut JNIEnv) -> jthrowable,
    pub ExceptionDescribe: fn(*mut JNIEnv),
    pub ExceptionClear: fn(*mut JNIEnv),
    pub FatalError: fn(*mut JNIEnv, *const c_char),

    pub PushLocalFrame: fn(*mut JNIEnv, jint) -> jint,
    pub PopLocalFrame: fn(*mut JNIEnv, jobject) -> jobject,

    pub NewGlobalRef: fn(*mut JNIEnv, jobject) -> jobject,
    pub DeleteGlobalRef: fn(*mut JNIEnv, jobject),
    pub DeleteLocalRef: fn(*mut JNIEnv, jobject),
    pub IsSameObject: fn(*mut JNIEnv, jobject, jobject) -> jboolean,
    pub NewLocalRef: fn(*mut JNIEnv, jobject) -> jobject,
    pub EnsureLocalCapacity: fn(*mut JNIEnv, jint) -> jint,

    pub AllocObject: fn(*mut JNIEnv, jclass) -> jobject,

    pub NewObject: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jobject,
    pub NewObjectV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jobject,
    pub NewObjectA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jobject,

    pub GetObjectClass: fn(*mut JNIEnv, jobject) -> jclass,
    pub IsInstanceOf: fn(*mut JNIEnv, jobject, jclass) -> jboolean,

    pub GetMethodID: fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jmethodID,

    pub CallObjectMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jobject,
    pub CallObjectMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jobject,
    pub CallObjectMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jobject,

    pub CallBooleanMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jboolean,
    pub CallBooleanMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jboolean,
    pub CallBooleanMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jboolean,

    pub CallByteMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jbyte,
    pub CallByteMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jbyte,
    pub CallByteMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jbyte,

    pub CallCharMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jchar,
    pub CallCharMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jchar,
    pub CallCharMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jchar,

    pub CallShortMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jshort,
    pub CallShortMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jshort,
    pub CallShortMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jshort,

    pub CallIntMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jint,
    pub CallIntMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jint,
    pub CallIntMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jint,

    pub CallLongMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jlong,
    pub CallLongMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jlong,
    pub CallLongMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jlong,

    pub CallFloatMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jfloat,
    pub CallFloatMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jfloat,
    pub CallFloatMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jfloat,

    pub CallDoubleMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...) -> jdouble,
    pub CallDoubleMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char) -> jdouble,
    pub CallDoubleMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue) -> jdouble,

    pub CallVoidMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jmethodID, ...),
    pub CallVoidMethodV: fn(*mut JNIEnv, jobject, jmethodID, *mut c_char),
    pub CallVoidMethodA: fn(*mut JNIEnv, jobject, jmethodID, *const jvalue),

    pub CallNonvirtualObjectMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jobject,
    pub CallNonvirtualObjectMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jobject,
    pub CallNonvirtualObjectMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jobject,

    pub CallNonvirtualBooleanMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jboolean,
    pub CallNonvirtualBooleanMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jboolean,
    pub CallNonvirtualBooleanMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jboolean,

    pub CallNonvirtualByteMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jbyte,
    pub CallNonvirtualByteMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jbyte,
    pub CallNonvirtualByteMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jbyte,

    pub CallNonvirtualCharMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jchar,
    pub CallNonvirtualCharMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jchar,
    pub CallNonvirtualCharMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jchar,

    pub CallNonvirtualShortMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jshort,
    pub CallNonvirtualShortMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jshort,
    pub CallNonvirtualShortMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jshort,

    pub CallNonvirtualIntMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jint,
    pub CallNonvirtualIntMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jint,
    pub CallNonvirtualIntMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jint,

    pub CallNonvirtualLongMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jlong,
    pub CallNonvirtualLongMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jlong,
    pub CallNonvirtualLongMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jlong,

    pub CallNonvirtualFloatMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jfloat,
    pub CallNonvirtualFloatMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jfloat,
    pub CallNonvirtualFloatMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jfloat,

    pub CallNonvirtualDoubleMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...) -> jdouble,
    pub CallNonvirtualDoubleMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char) -> jdouble,
    pub CallNonvirtualDoubleMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue) -> jdouble,

    pub CallNonvirtualVoidMethod: unsafe extern "C" fn(*mut JNIEnv, jobject, jclass, jmethodID, ...),
    pub CallNonvirtualVoidMethodV: fn(*mut JNIEnv, jobject, jclass, jmethodID, *mut c_char),
    pub CallNonvirtualVoidMethodA: fn(*mut JNIEnv, jobject, jclass, jmethodID, *const jvalue),

    pub GetFieldID: fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jfieldID,

    pub GetObjectField: fn(*mut JNIEnv, jobject, jfieldID) -> jobject,
    pub GetBooleanField: fn(*mut JNIEnv, jobject, jfieldID) -> jboolean,
    pub GetByteField: fn(*mut JNIEnv, jobject, jfieldID) -> jbyte,
    pub GetCharField: fn(*mut JNIEnv, jobject, jfieldID) -> jchar,
    pub GetShortField: fn(*mut JNIEnv, jobject, jfieldID) -> jshort,
    pub GetIntField: fn(*mut JNIEnv, jobject, jfieldID) -> jint,
    pub GetLongField: fn(*mut JNIEnv, jobject, jfieldID)
    -> jlong,
    pub GetFloatField: fn(*mut JNIEnv, jobject, jfieldID) -> jfloat,
    pub GetDoubleField: fn(*mut JNIEnv, jobject, jfieldID) -> jdouble,

    pub SetObjectField: fn(*mut JNIEnv, jobject, jfieldID, jobject),
    pub SetBooleanField: fn(*mut JNIEnv, jobject, jfieldID, jboolean),
    pub SetByteField: fn(*mut JNIEnv, jobject, jfieldID, jbyte),
    pub SetCharField: fn(*mut JNIEnv, jobject, jfieldID, jchar),
    pub SetShortField: fn(*mut JNIEnv, jobject, jfieldID, jshort),
    pub SetIntField: fn(*mut JNIEnv, jobject, jfieldID, jint),
    pub SetLongField: fn(*mut JNIEnv, jobject, jfieldID, jlong),
    pub SetFloatField: fn(*mut JNIEnv, jobject, jfieldID, jfloat),
    pub SetDoubleField: fn(*mut JNIEnv, jobject, jfieldID, jdouble),

    pub GetStaticMethodID: fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jmethodID,

    pub CallStaticObjectMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jobject,
    pub CallStaticObjectMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jobject,
    pub CallStaticObjectMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jobject,

    pub CallStaticBooleanMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jboolean,
    pub CallStaticBooleanMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jboolean,
    pub CallStaticBooleanMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jboolean,

    pub CallStaticByteMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jbyte,
    pub CallStaticByteMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jbyte,
    pub CallStaticByteMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jbyte,

    pub CallStaticCharMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jchar,
    pub CallStaticCharMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jchar,
    pub CallStaticCharMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jchar,

    pub CallStaticShortMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jshort,
    pub CallStaticShortMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jshort,
    pub CallStaticShortMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jshort,

    pub CallStaticIntMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jint,
    pub CallStaticIntMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jint,
    pub CallStaticIntMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jint,

    pub CallStaticLongMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jlong,
    pub CallStaticLongMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jlong,
    pub CallStaticLongMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jlong,

    pub CallStaticFloatMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jfloat,
    pub CallStaticFloatMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jfloat,
    pub CallStaticFloatMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jfloat,

    pub CallStaticDoubleMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...) -> jdouble,
    pub CallStaticDoubleMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char) -> jdouble,
    pub CallStaticDoubleMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue) -> jdouble,

    pub CallStaticVoidMethod: unsafe extern "C" fn(*mut JNIEnv, jclass, jmethodID, ...),
    pub CallStaticVoidMethodV: fn(*mut JNIEnv, jclass, jmethodID, *mut c_char),
    pub CallStaticVoidMethodA: fn(*mut JNIEnv, jclass, jmethodID, *const jvalue),

    pub GetStaticFieldID: fn(*mut JNIEnv, jclass, *const c_char, *const c_char) -> jfieldID,
    pub GetStaticObjectField: fn(*mut JNIEnv, jclass, jfieldID) -> jobject,
    pub GetStaticBooleanField: fn(*mut JNIEnv, jclass, jfieldID) -> jboolean,
    pub GetStaticByteField: fn(*mut JNIEnv, jclass, jfieldID) -> jbyte,
    pub GetStaticCharField: fn(*mut JNIEnv, jclass, jfieldID) -> jchar,
    pub GetStaticShortField: fn(*mut JNIEnv, jclass, jfieldID) -> jshort,
    pub GetStaticIntField: fn(*mut JNIEnv, jclass, jfieldID) -> jint,
    pub GetStaticLongField: fn(*mut JNIEnv, jclass, jfieldID) -> jlong,
    pub GetStaticFloatField: fn(*mut JNIEnv, jclass, jfieldID) -> jfloat,
    pub GetStaticDoubleField: fn(*mut JNIEnv, jclass, jfieldID) -> jdouble,

    pub SetStaticObjectField: fn(*mut JNIEnv, jclass, jfieldID, jobject),
    pub SetStaticBooleanField: fn(*mut JNIEnv, jclass, jfieldID, jboolean),
    pub SetStaticByteField: fn(*mut JNIEnv, jclass, jfieldID, jbyte),
    pub SetStaticCharField: fn(*mut JNIEnv, jclass, jfieldID, jchar),
    pub SetStaticShortField: fn(*mut JNIEnv, jclass, jfieldID, jshort),
    pub SetStaticIntField: fn(*mut JNIEnv, jclass, jfieldID, jint),
    pub SetStaticLongField: fn(*mut JNIEnv, jclass, jfieldID, jlong),
    pub SetStaticFloatField: fn(*mut JNIEnv, jclass, jfieldID, jfloat),
    pub SetStaticDoubleField: fn(*mut JNIEnv, jclass, jfieldID, jdouble),

    pub NewString: fn(*mut JNIEnv, *const jchar, jsize) -> jstring,
    pub GetStringLength: fn(*mut JNIEnv, jstring) -> jsize,
    pub GetStringChars: fn(*mut JNIEnv, jstring, *mut jboolean) -> *const jchar,
    pub ReleaseStringChars: fn(*mut JNIEnv, jstring, *const jchar),

    pub NewStringUTF: fn(*mut JNIEnv, *const c_char) -> jstring,
    pub GetStringUTFLength: fn(*mut JNIEnv, jstring) -> jsize,
    pub GetStringUTFChars: fn(*mut JNIEnv, jstring, *mut jboolean) -> *const c_char,
    pub ReleaseStringUTFChars: fn(*mut JNIEnv, jstring, *const c_char),

    pub GetArrayLength: fn(*mut JNIEnv, jarray) -> jsize,

    pub NewObjectArray: fn(*mut JNIEnv, jsize, jclass, jobject) -> jobjectArray,
    pub GetObjectArrayElement: fn(*mut JNIEnv, jobjectArray, jsize) -> jobject,
    pub SetObjectArrayElement: fn(*mut JNIEnv, jobjectArray, jsize, jobject),

    pub NewBooleanArray: fn(*mut JNIEnv, jsize) -> jbooleanArray,
    pub NewByteArray: fn(*mut JNIEnv, jsize) -> jbyteArray,
    pub NewCharArray: fn(*mut JNIEnv, jsize) -> jcharArray,
    pub NewShortArray: fn(*mut JNIEnv, jsize) -> jshortArray,
    pub NewIntArray: fn(*mut JNIEnv, jsize) -> jintArray,
    pub NewLongArray: fn(*mut JNIEnv, jsize) -> jlongArray,
    pub NewFloatArray: fn(*mut JNIEnv, jsize) -> jfloatArray,
    pub NewDoubleArray: fn(*mut JNIEnv, jsize) -> jdoubleArray,

    pub GetBooleanArrayElements: fn(*mut JNIEnv, jbooleanArray, *mut jboolean) -> *mut jboolean,
    pub GetByteArrayElements: fn(*mut JNIEnv, jbyteArray, *mut jboolean) -> *mut jbyte,
    pub GetCharArrayElements: fn(*mut JNIEnv, jcharArray, *mut jboolean) -> *mut jchar,
    pub GetShortArrayElements: fn(*mut JNIEnv, jshortArray, *mut jboolean) -> *mut jshort,
    pub GetIntArrayElements: fn(*mut JNIEnv, jintArray, *mut jboolean) -> *mut jint,
    pub GetLongArrayElements: fn(*mut JNIEnv, jlongArray, *mut jboolean) -> *mut jlong,
    pub GetFloatArrayElements: fn(*mut JNIEnv, jfloatArray, *mut jboolean) -> *mut jfloat,
    pub GetDoubleArrayElements: fn(*mut JNIEnv, jdoubleArray, *mut jboolean) -> *mut jdouble,

    pub ReleaseBooleanArrayElements: fn(*mut JNIEnv, jbooleanArray, *mut jboolean, jint),
    pub ReleaseByteArrayElements: fn(*mut JNIEnv, jbyteArray, *mut jbyte, jint),
    pub ReleaseCharArrayElements: fn(*mut JNIEnv, jcharArray, *mut jchar, jint),
    pub ReleaseShortArrayElements: fn(*mut JNIEnv, jshortArray, *mut jshort, jint),
    pub ReleaseIntArrayElements: fn(*mut JNIEnv, jintArray, *mut jint, jint),
    pub ReleaseLongArrayElements: fn(*mut JNIEnv, jlongArray, *mut jlong, jint),
    pub ReleaseFloatArrayElements: fn(*mut JNIEnv, jfloatArray, *mut jfloat, jint),
    pub ReleaseDoubleArrayElements: fn(*mut JNIEnv, jdoubleArray, *mut jdouble, jint),

    pub GetBooleanArrayRegion: fn(*mut JNIEnv, jbooleanArray, jsize, jsize, *mut jboolean),
    pub GetByteArrayRegion: fn(*mut JNIEnv, jbyteArray, jsize, jsize, *mut jbyte),
    pub GetCharArrayRegion: fn(*mut JNIEnv, jcharArray, jsize, jsize, *mut jchar),
    pub GetShortArrayRegion: fn(*mut JNIEnv, jshortArray, jsize, jsize, *mut jshort),
    pub GetIntArrayRegion: fn(*mut JNIEnv, jintArray, jsize, jsize, *mut jint),
    pub GetLongArrayRegion: fn(*mut JNIEnv, jlongArray, jsize, jsize, *mut jlong),
    pub GetFloatArrayRegion: fn(*mut JNIEnv, jfloatArray, jsize, jsize, *mut jfloat),
    pub GetDoubleArrayRegion: fn(*mut JNIEnv, jdoubleArray, jsize, jsize, *mut jdouble),

    pub SetBooleanArrayRegion: fn(*mut JNIEnv, jbooleanArray, jsize, jsize, *const jboolean),
    pub SetByteArrayRegion: fn(*mut JNIEnv, jbyteArray, jsize, jsize, *const jbyte),
    pub SetCharArrayRegion: fn(*mut JNIEnv, jcharArray, jsize, jsize, *const jchar),
    pub SetShortArrayRegion: fn(*mut JNIEnv, jshortArray, jsize, jsize, *const jshort),
    pub SetIntArrayRegion: fn(*mut JNIEnv, jintArray, jsize, jsize, *const jint),
    pub SetLongArrayRegion: fn(*mut JNIEnv, jlongArray, jsize, jsize, *const jlong),
    pub SetFloatArrayRegion: fn(*mut JNIEnv, jfloatArray, jsize, jsize, *const jfloat),
    pub SetDoubleArrayRegion: fn(*mut JNIEnv, jdoubleArray, jsize, jsize, *const jdouble),

    pub RegisterNatives: fn(*mut JNIEnv, jclass, *const JNINativeMethod, jint) -> jint,
    pub UnregisterNatives: fn(*mut JNIEnv, jclass) -> jint,

    pub MonitorEnter: fn(*mut JNIEnv, jobject) -> jint,
    pub MonitorExit: fn(*mut JNIEnv, jobject) -> jint,

    pub GetJavaVM: fn(*mut JNIEnv, *mut *mut JavaVM) -> jint,

    pub GetStringRegion: fn(*mut JNIEnv, jstring, jsize, jsize, *mut jchar),
    pub GetStringUTFRegion: fn(*mut JNIEnv, jstring, jsize, jsize, *mut c_char),

    pub GetPrimitiveArrayCritical: fn(*mut JNIEnv, jarray, *mut jboolean) -> *mut c_void,
    pub ReleasePrimitiveArrayCritical: fn(*mut JNIEnv, jarray, *mut c_void, jint),

    pub GetStringCritical: fn(*mut JNIEnv, jstring, *mut jboolean) -> *const c_char,
    pub ReleaseStringCritical: fn(*mut JNIEnv, jstring, *const c_char),

    pub NewWeakGlobalRef: fn(*mut JNIEnv, jobject) -> jweak,
    pub DeleteWeakGlobalRef: fn(*mut JNIEnv, jweak),

    pub ExceptionCheck: fn(*mut JNIEnv) -> jboolean,

    pub NewDirectByteBuffer: fn(*mut JNIEnv, *mut c_void, jlong) -> jobject,
    pub GetDirectBufferAddress: fn(*mut JNIEnv, jobject) -> *mut c_void,
    pub GetDirectBufferCapacity: fn(*mut JNIEnv, jobject) -> jlong,

    pub GetObjectRefType: fn(*mut JNIEnv, jobject) -> jobjectRefType,

    pub GetModule: fn(*mut JNIEnv, jobject) -> jobject,

    pub IsVirtualThread: fn(*mut JNIEnv, jobject) -> jboolean,
}

#[repr(C)]
pub struct JavaVMOption {
    optionString: *mut c_char,
    extraInfo: *mut c_void,
}

#[repr(C)]
pub struct JavaVMInitArgs {
    pub version: jint,
    pub nOptions: jint,
    pub options: *mut JavaVMOption,
    pub ignoreUnrecognized: jboolean,
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

    pub DestroyJavaVM: fn(*mut JavaVM) -> jint,
    pub AttachCurrentThread: fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
    pub DetachCurrentThread: fn(*mut JavaVM) -> jint,
    pub GetEnv: fn(*mut JavaVM, *mut *mut c_void, jint) -> jint,
    pub AttachCurrentThreadAsDaemon: fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
}

pub type JavaVM = *mut JNIInvokeInterface;

pub const JNI_VERSION: jint = 0x00150000;
