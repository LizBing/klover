/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
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

#[repr(C)]
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
    pub fn_ptr: *mut c_void,
}

pub type JNINativeInterface = c_void;
pub type JNIEnv = *mut JNINativeInterface;

#[repr(C)]
pub struct JavaVMOption {
    opt_str: *mut c_char,
    extra_info: *mut c_void,
}

#[repr(C)]
pub struct JavaVMInitArgs {
    pub version: jint,
    pub n_opts: jint,
    pub options: *mut JavaVMOption,
    pub ign_unrecognized: jboolean,
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

    pub destroy_java_vm: fn(*mut JavaVM) -> jint,
    pub attach_current_thread: fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
    pub detach_current_thread: fn(*mut JavaVM) -> jint,
    pub get_env: fn(*mut JavaVM, *mut *mut c_void, jint) -> jint,
    pub attach_current_thread_daemon: fn(*mut JavaVM, *mut *mut c_void, *mut c_void) -> jint,
}

pub type JavaVM = *mut JNIInvokeInterface;

pub const JNI_VERSION: jint = 0x00150000;
