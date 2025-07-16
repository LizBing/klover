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

use std::{ffi::{c_char, c_void, CStr}, ptr::null_mut};

use crate::prims::jni::{JavaVMInitArgs};

use super::jni::{jarray, jboolean, jbyte, jclass, jint, jobject, jsize, JNIEnv, JNIInvokeInterface, JavaVM, JNI_OK, JNI_VERSION};
use super::jni::{jmethodID, jfieldID, jthrowable, jchar, jshort, jlong, jfloat, jdouble, jstring, jweak, jobjectRefType, JNINativeMethod, jvalue}; // Added jvalue

#[no_mangle]
extern "C" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
    debug_assert!(args != null_mut(), "should not be null.");

    let init_args = unsafe { &mut *(args as *mut JavaVMInitArgs) };
    // todo: verify the version.

    init_args.version = JNI_VERSION;
    init_args.n_opts = 0;
    init_args.options = null_mut();
    init_args.ign_unrecognized = 1;

    JNI_OK
}

extern "C" { fn get_jni_env() -> JNIEnv; }

#[no_mangle]
unsafe extern "C" fn JNI_CreateJavaVM(pvm: *mut *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "C" fn JNI_GetCreatedJavaVMs(vm_buf: *mut *mut JavaVM, len: jsize, nvms: *mut jsize) -> jint {
    unimplemented!()
}

#[no_mangle]
extern "C" fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut c_void) -> jint {
    JNI_VERSION
}

#[no_mangle]
extern "C" fn JNI_OnUnload(vm: *mut JavaVM, reserved: *mut c_void) -> jint {
    JNI_VERSION
}

// Methods in JNIInvokeInterface

fn destroy_java_vm(vm: *mut JavaVM) -> jint {
    unimplemented!()
}

fn attach_current_thread(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    unimplemented!()
}

fn detach_current_thread(vm: *mut JavaVM) -> jint {
    unimplemented!()
}

fn get_env(vm: *mut JavaVM, penv: *mut *mut c_void, version: jint) -> jint {
    unimplemented!()
}

fn attach_current_thread_as_daemon(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    unimplemented!()
}

static mut JNI_INVOKE_INTERFACE: JNIInvokeInterface = JNIInvokeInterface {
    reserved0: null_mut(),
    reserved1: null_mut(),
    reserved2: null_mut(),

    destroy_java_vm,
    attach_current_thread,
    detach_current_thread,
    get_env,
    attach_current_thread_daemon: attach_current_thread_as_daemon
};

// Methods in JNINativeInterface_

const DUMMY_OBJ: jobject = 1 as _;
const DUMMY_FIELD: jfieldID = 2 as _;
const DUMMY_MTHD: jmethodID = 3 as _;

static mut DUMMY_SLOT: usize = 0;
static mut DUMMY_VEC: Vec<i8> = Vec::new();

fn get_version(_env: *mut JNIEnv) -> jint {
    JNI_VERSION
}

fn define_class(_env: *mut JNIEnv, _name: *const c_char, _loader: jobject, _buf: *const jbyte, _buf_len: jsize) -> jclass {
    DUMMY_OBJ
}

fn find_class(_env: *mut JNIEnv, _name: *const c_char) -> jclass {
    DUMMY_OBJ
}

fn get_super_class(_env: *mut JNIEnv, _clazz: jclass) -> jclass {
    DUMMY_OBJ
}

fn is_assignable_from(_env: *mut JNIEnv, _clazz1: jclass, _clazz2: jclass) -> jboolean {
    1
}

fn from_reflected_method(_env: *mut JNIEnv, _method: jobject) -> jmethodID {
    DUMMY_MTHD
}

fn from_reflected_field(_env: *mut JNIEnv, _field: jobject) -> jfieldID {
    DUMMY_FIELD
}

fn to_reflected_method(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _is_static: jboolean) -> jobject {
    DUMMY_OBJ
}

fn to_reflected_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _is_static: jboolean) -> jobject {
    DUMMY_OBJ
}

fn throw(_env: *mut JNIEnv, _obj: jthrowable) -> jint {
    JNI_OK
}

fn exception_occurred(_env: *mut JNIEnv) -> jthrowable {
    DUMMY_OBJ
}

fn exception_describe(_env: *mut JNIEnv) {
    eprintln!("no exception occurred")
}

fn exception_clear(_env: *mut JNIEnv) { }

fn fatal_error(_env: *mut JNIEnv, msg: *const c_char) {
    unsafe { println!("fatal error: {}", CStr::from_ptr(msg).to_str().unwrap()) }
}

fn push_local_frame(_env: *mut JNIEnv, _capacity: jint) -> jint {
    JNI_OK
}

fn pop_local_frame(_env: *mut JNIEnv, _result: jobject) -> jobject {
    DUMMY_OBJ
}

fn new_global_ref(_env: *mut JNIEnv, _obj: jobject) -> jobject {
    DUMMY_OBJ
}

fn delete_global_ref(_env: *mut JNIEnv, _global_ref: jobject) { }

fn delete_local_ref(_env: *mut JNIEnv, _local_ref: jobject) { }

fn is_same_object(_env: *mut JNIEnv, _obj1: jobject, _obj2: jobject) -> jboolean {
    1
}

fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    JNI_OK
}

fn alloc_object(_env: *mut JNIEnv, _clazz: jclass) -> jobject {
    DUMMY_OBJ
}

fn get_object_class(_env: *mut JNIEnv, _obj: jobject) -> jclass {
    DUMMY_OBJ
}

fn is_instance_of(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass) -> jboolean {
    1
}

fn get_method_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jmethodID {
    DUMMY_MTHD
}

fn get_field_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jfieldID {
    DUMMY_FIELD
}

fn get_object_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jobject {
    DUMMY_OBJ
}

fn get_boolean_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jboolean {
    unimplemented!()
}

fn get_byte_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jbyte {
    1
}

fn get_char_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jchar {
    1
}

fn get_short_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jshort {
    1
}

fn get_int_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jint {
    1
}

fn get_long_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jlong {
    1
}

fn get_float_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jfloat {
    1.0
}

fn get_double_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jdouble {
    1.0
}

fn set_object_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jobject) { }

fn set_boolean_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jboolean) { }

fn set_byte_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jbyte) { }

fn set_char_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jchar) { }

fn set_short_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jshort) { }

fn set_int_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jint) { }

fn set_long_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jlong) { }

fn set_float_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jfloat) { }

fn set_double_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jdouble) { }

fn monitor_enter(_env: *mut JNIEnv, _obj: jobject) -> jint {
    JNI_OK
}

fn monitor_exit(_env: *mut JNIEnv, _obj: jobject) -> jint {
    JNI_OK
}

fn get_array_length(_env: *mut JNIEnv, _array: jobject) -> jsize {
    1
}

fn new_object_array(_env: *mut JNIEnv, _length: jsize, _clazz: jclass, _init: jobject) -> jobject {
    DUMMY_OBJ
}

fn get_object_array_element(_env: *mut JNIEnv, _array: jobject, _index: jsize) -> jobject {
    DUMMY_OBJ
}

fn set_object_array_element(_env: *mut JNIEnv, _array: jobject, _index: jsize, _value: jobject) { }

fn new_boolean_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_byte_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_char_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_short_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_int_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_long_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_float_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn new_double_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    DUMMY_OBJ
}

fn get_boolean_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jboolean {
    unsafe { DUMMY_SLOT as _ }
}

fn get_byte_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jbyte {
    unsafe { DUMMY_SLOT as _ }
}

fn get_char_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jchar {
    unsafe { DUMMY_SLOT as _ }
}

fn get_short_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jshort {
    unsafe { DUMMY_SLOT as _ }
}

fn get_int_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jint {
    unsafe { DUMMY_SLOT as _ }
}

fn get_long_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jlong {
    unsafe { DUMMY_SLOT as _ }
}

fn get_float_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jfloat {
    unsafe { DUMMY_SLOT as _ }
}

fn get_double_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jdouble {
    unsafe { DUMMY_SLOT as _ }
}

fn release_boolean_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jboolean, _mode: jint) { }

fn release_byte_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jbyte, _mode: jint) { }

fn release_char_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jchar, _mode: jint) { }

fn release_short_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jshort, _mode: jint) { }

fn release_int_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jint, _mode: jint) { }

fn release_long_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jlong, _mode: jint) { }

fn release_float_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jfloat, _mode: jint) { }

fn release_double_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jdouble, _mode: jint) { }

fn get_boolean_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jboolean) { }

fn get_byte_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jbyte) { }

fn get_char_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jchar) { }

fn get_short_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jshort) { }

fn get_int_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jint) { }

fn get_long_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jlong) { }

fn get_float_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jfloat) { }

fn get_double_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jdouble) { }

fn set_boolean_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jboolean) { }

fn set_byte_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jbyte) { }

fn set_char_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jchar) { }

fn set_short_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jshort) { }

fn set_int_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jint) { }

fn set_long_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jlong) { }

fn set_float_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jfloat) { }

fn set_double_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jdouble) { }

fn throw_new(_env: *mut JNIEnv, _clazz: jclass, _msg: *const c_char) -> jint {
    JNI_OK
}

fn new_local_ref(_env: *mut JNIEnv, _obj: jobject) -> jobject {
    DUMMY_OBJ
}

fn get_static_method_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jmethodID {
    DUMMY_MTHD
}

fn get_static_field_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jfieldID {
    DUMMY_FIELD
}

fn get_static_object_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jobject { DUMMY_OBJ }
fn get_static_boolean_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jboolean { 1 }
fn get_static_byte_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jbyte { 1 }
fn get_static_char_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jchar { 1 }
fn get_static_short_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jshort { 1 }
fn get_static_int_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jint { 1 }
fn get_static_long_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jlong { 1 }
fn get_static_float_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jfloat { 1.0 }
fn get_static_double_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jdouble { 1.0 }

fn set_static_object_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jobject) { }
fn set_static_boolean_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jboolean) { }
fn set_static_byte_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jbyte) { }
fn set_static_char_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jchar) { }
fn set_static_short_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jshort) { }
fn set_static_int_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jint) { }
fn set_static_long_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jlong) { }
fn set_static_float_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jfloat) { }
fn set_static_double_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jdouble) { }

fn new_string(_env: *mut JNIEnv, _unicode_chars: *const jchar, _len: jsize) -> jstring {
    DUMMY_OBJ
}

fn get_string_length(_env: *mut JNIEnv, _string: jstring) -> jsize {
    1
}

fn get_string_chars(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const jchar {
    [1, 2, 3, 0].as_ptr()
}

fn release_string_chars(_env: *mut JNIEnv, _string: jstring, _chars: *const jchar) { }

fn new_string_utf(_env: *mut JNIEnv, _bytes: *const c_char) -> jstring {
    DUMMY_OBJ
}

fn get_string_utf_length(_env: *mut JNIEnv, _string: jstring) -> jsize {
    1
}

fn get_string_utf_chars(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const c_char {
    [1, 2, 3, 0].as_ptr()
}

fn release_string_utf_chars(_env: *mut JNIEnv, _string: jstring, _utf: *const c_char) { }

fn register_natives(_env: *mut JNIEnv, _clazz: jclass, _methods: *const JNINativeMethod, _n_methods: jint) -> jint {
    JNI_OK
}

fn unregister_natives(_env: *mut JNIEnv, _clazz: jclass) -> jint {
    JNI_OK
}

fn get_java_vm(_env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    unimplemented!()
}

fn get_string_region(_env: *mut JNIEnv, _str: jstring, _start: jsize, _len: jsize, _buf: *mut jchar) { }

fn get_string_utf_region(_env: *mut JNIEnv, _str: jstring, _start: jsize, _len: jsize, _buf: *mut c_char) { }

fn get_pri_arr_helper() -> *mut c_void {
    unsafe {
        if DUMMY_VEC.capacity() == 0 {
            DUMMY_VEC.resize(4096, 1);
        }

        DUMMY_VEC[4095] = 0;

        DUMMY_VEC.as_mut_ptr() as _
    }
}

fn get_primitive_array_critical(_env: *mut JNIEnv, _array: jarray, _is_copy: *mut jboolean) -> *mut c_void {
    unsafe {
        *_is_copy = 0;
    }

    get_pri_arr_helper()
}

fn release_primitive_array_critical(_env: *mut JNIEnv, _array: jarray, _carray: *mut c_void, _mode: jint) { }

fn get_string_critical(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const c_char {
    get_pri_arr_helper() as _
}

fn release_string_critical(_env: *mut JNIEnv, _string: jstring, _cstring: *const c_char) { }

fn new_weak_global_ref(_env: *mut JNIEnv, _obj: jobject) -> jweak {
    DUMMY_OBJ
}

fn delete_weak_global_ref(_env: *mut JNIEnv, _obj: jweak) { }

fn exception_check(_env: *mut JNIEnv) -> jboolean {
    1
}

fn new_direct_byte_buffer(_env: *mut JNIEnv, _address: *mut c_void, _capacity: jlong) -> jobject {
    DUMMY_OBJ
}

fn get_direct_buffer_address(_env: *mut JNIEnv, _buf: jobject) -> *mut c_void {
    get_pri_arr_helper()
}

fn get_direct_buffer_capacity(_env: *mut JNIEnv, _buf: jobject) -> jlong {
    1
}

fn get_object_ref_type(_env: *mut JNIEnv, _obj: jobject) -> jobjectRefType {
    jobjectRefType::JNILocalRefType
}

fn get_module(_env: *mut JNIEnv, _clazz: jclass) -> jobject {
    DUMMY_OBJ
}

fn is_virtual_thread(_env: *mut JNIEnv, _obj: jobject) -> jboolean {
    0
}

// Stubs for C-variadic JNI functions
extern "C" {
    fn new_object(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jobject;
        // Call<Type>Method
    fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jobject;
    fn call_boolean_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jboolean;
    fn call_byte_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jbyte;
    fn call_char_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jchar;
    fn call_short_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jshort;
    fn call_int_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jint;
    fn call_long_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jlong;
    fn call_float_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jfloat;
    fn call_double_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...) -> jdouble;
    fn call_void_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, ...);

    // CallNonvirtual<Type>Method
    fn call_nonvirtual_object_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jobject;
    fn call_nonvirtual_boolean_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jboolean;
    fn call_nonvirtual_byte_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jbyte;
    fn call_nonvirtual_char_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jchar;
    fn call_nonvirtual_short_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jshort;
    fn call_nonvirtual_int_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jint;
    fn call_nonvirtual_long_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jlong;
    fn call_nonvirtual_float_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jfloat;
    fn call_nonvirtual_double_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...) -> jdouble;
    fn call_nonvirtual_void_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, ...);

    // CallStatic<Type>Method
    fn call_static_object_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jobject;
    fn call_static_boolean_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jboolean;
    fn call_static_byte_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jbyte;
    fn call_static_char_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jchar;
    fn call_static_short_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jshort;
    fn call_static_int_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jint;
    fn call_static_long_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jlong;
    fn call_static_float_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jfloat;
    fn call_static_double_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...) -> jdouble;
    fn call_static_void_method(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, ...);
}

fn call_boolean_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jboolean { 0 }
fn call_byte_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jbyte { 0 }
fn call_char_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jchar { 0 }
fn call_short_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jshort { 0 }
fn call_int_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jint { 0 }
fn call_long_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jlong { 0 }
fn call_float_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jfloat { 0.0 }
fn call_double_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jdouble { 0.0 }
fn call_object_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jobject { DUMMY_OBJ }
fn call_string_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jstring { DUMMY_OBJ }
fn call_array_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jarray { DUMMY_OBJ }
fn call_throwable_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) -> jthrowable { DUMMY_OBJ }
fn call_void_method_v(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *mut i8) {}

fn call_boolean_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jboolean { 0 }
fn call_byte_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jbyte { 0 }
fn call_char_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jchar { 0 }
fn call_short_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jshort { 0 }
fn call_int_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jint { 0 }
fn call_long_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jlong { 0 }
fn call_float_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jfloat { 0.0 }
fn call_double_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jdouble { 0.0 }
fn call_object_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jobject { DUMMY_OBJ }
fn call_string_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jstring { DUMMY_OBJ }
fn call_array_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jarray { DUMMY_OBJ }
fn call_throwable_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) -> jthrowable { DUMMY_OBJ }
fn call_void_method_a(_env: *mut JNIEnv, _obj: jobject, _method_id: jmethodID, _args: *const jvalue) {}

fn call_static_boolean_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jboolean { 0 }
fn call_static_byte_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jbyte { 0 }
fn call_static_char_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jchar { 0 }
fn call_static_short_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jshort { 0 }
fn call_static_int_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jint { 0 }
fn call_static_long_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jlong { 0 }
fn call_static_float_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jfloat { 0.0 }
fn call_static_double_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jdouble { 0.0 }
fn call_static_object_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jobject { DUMMY_OBJ }
fn call_static_string_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jstring { DUMMY_OBJ }
fn call_static_array_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jarray { DUMMY_OBJ }
fn call_static_throwable_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jthrowable { DUMMY_OBJ }
fn call_static_void_method_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) {}

fn call_static_boolean_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jboolean { 0 }
fn call_static_byte_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jbyte { 0 }
fn call_static_char_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jchar { 0 }
fn call_static_short_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jshort { 0 }
fn call_static_int_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jint { 0 }
fn call_static_long_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jlong { 0 }
fn call_static_float_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jfloat { 0.0 }
fn call_static_double_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jdouble { 0.0 }
fn call_static_object_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jobject { DUMMY_OBJ }
fn call_static_string_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jstring { DUMMY_OBJ }
fn call_static_array_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jarray { DUMMY_OBJ }
fn call_static_throwable_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jthrowable { DUMMY_OBJ }
fn call_static_void_method_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) {}

fn call_nonvirtual_boolean_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jboolean { 0 }
fn call_nonvirtual_byte_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jbyte { 0 }
fn call_nonvirtual_char_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jchar { 0 }
fn call_nonvirtual_short_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jshort { 0 }
fn call_nonvirtual_int_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jint { 0 }
fn call_nonvirtual_long_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jlong { 0 }
fn call_nonvirtual_float_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jfloat { 0.0 }
fn call_nonvirtual_double_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jdouble { 0.0 }
fn call_nonvirtual_object_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jobject { DUMMY_OBJ }
fn call_nonvirtual_string_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jstring { DUMMY_OBJ }
fn call_nonvirtual_array_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jarray { DUMMY_OBJ }
fn call_nonvirtual_throwable_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jthrowable { DUMMY_OBJ }
fn call_nonvirtual_void_method_v(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) {}

fn call_nonvirtual_boolean_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jboolean { 0 }
fn call_nonvirtual_byte_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jbyte { 0 }
fn call_nonvirtual_char_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jchar { 0 }
fn call_nonvirtual_short_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jshort { 0 }
fn call_nonvirtual_int_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jint { 0 }
fn call_nonvirtual_long_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jlong { 0 }
fn call_nonvirtual_float_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jfloat { 0.0 }
fn call_nonvirtual_double_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jdouble { 0.0 }
fn call_nonvirtual_object_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jobject { DUMMY_OBJ }
fn call_nonvirtual_string_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jstring { DUMMY_OBJ }
fn call_nonvirtual_array_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jarray { DUMMY_OBJ }
fn call_nonvirtual_throwable_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jthrowable { DUMMY_OBJ }
fn call_nonvirtual_void_method_a(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) {}

fn new_object_v(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *mut i8) -> jobject {
    DUMMY_OBJ
}

fn new_object_a(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _args: *const jvalue) -> jobject {
    DUMMY_OBJ
}

