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

use std::{ffi::{c_char, c_void}, ptr::null_mut, sync::atomic::{AtomicI32, Ordering}};

use crate::prims::jni::{JavaVMInitArgs, JNI_ERR};
use crate::prims::jni::JNINativeInterface;

use super::jni::{jarray, jboolean, jbyte, jclass, jint, jobject, jsize, JNIEnv, JNIInvokeInterface, JavaVM, JNI_OK, JNI_VERSION};
use super::jni::{jmethodID, jfieldID, jthrowable, jchar, jshort, jlong, jfloat, jdouble, jstring, jweak, jobjectRefType, JNINativeMethod, jvalue}; // Added jvalue

enum VMCreationState {
    NotCreated = 0,
    InProgress = 1,
    Complete = 2
}

static VM_CREATED: AtomicI32 = AtomicI32::new(VMCreationState::NotCreated as i32);
static mut MAIN_VM: JavaVM = unsafe { &mut JNI_INVOKE_INTERFACE };

#[no_mangle]
extern "C" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
    debug_assert!(args != null_mut(), "should not be null.");

    let init_args = unsafe { &mut *(args as *mut JavaVMInitArgs) };
    // todo: verify the version.

    init_args.version = JNI_VERSION;
    init_args.nOptions = 0;
    init_args.options = null_mut();
    init_args.ignoreUnrecognized = 1;

    JNI_OK
}

#[no_mangle]
extern "C" fn JNI_CreateJavaVM(pvm: *mut *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    println!("Hello!");

    JNI_ERR
}

#[no_mangle]
unsafe extern "C" fn JNI_GetCreatedJavaVMs(vm_buf: *mut *mut JavaVM, len: jsize, nvms: *mut jsize) -> jint {
    if VM_CREATED.load(Ordering::SeqCst) == VMCreationState::Complete as i32 {
        if nvms != null_mut() { *nvms = 1 }
        if len > 0 { *vm_buf = &mut MAIN_VM }
    } else {
        if nvms != null_mut() { *nvms = 0 }
    }

    JNI_OK
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

extern "C" fn destroy_java_vm(vm: *mut JavaVM) -> jint {
    JNI_OK
}

extern "C" fn attach_current_thread(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    JNI_ERR
}

extern "C" fn detach_current_thread(vm: *mut JavaVM) -> jint {
    JNI_ERR
}

extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut c_void, version: jint) -> jint {
    JNI_ERR
}

extern "C" fn attach_current_thread_as_daemon(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    JNI_ERR
}

static mut JNI_INVOKE_INTERFACE: JNIInvokeInterface = JNIInvokeInterface {
    reserved0: null_mut(),
    reserved1: null_mut(),
    reserved2: null_mut(),

    DestroyJavaVM: destroy_java_vm,
    AttachCurrentThread: attach_current_thread,
    DetachCurrentThread: detach_current_thread,
    GetEnv: get_env,
    AttachCurrentThreadAsDaemon: attach_current_thread_as_daemon
};

// Methods in JNINativeInterface_

extern "C" fn get_version(_env: *mut JNIEnv) -> jint {
    unimplemented!()
}

extern "C" fn define_class(_env: *mut JNIEnv, _name: *const c_char, _loader: jobject, _buf: *const jbyte, _buf_len: jsize) -> jclass {
    unimplemented!()
}

extern "C" fn find_class(_env: *mut JNIEnv, _name: *const c_char) -> jclass {
    unimplemented!()
}

extern "C" fn get_super_class(_env: *mut JNIEnv, _clazz: jclass) -> jclass {
    unimplemented!()
}

extern "C" fn is_assignable_from(_env: *mut JNIEnv, _clazz1: jclass, _clazz2: jclass) -> jboolean {
    unimplemented!()
}

extern "C" fn from_reflected_method(_env: *mut JNIEnv, _method: jobject) -> jmethodID {
    unimplemented!()
}

extern "C" fn from_reflected_field(_env: *mut JNIEnv, _field: jobject) -> jfieldID {
    unimplemented!()
}

extern "C" fn to_reflected_method(_env: *mut JNIEnv, _clazz: jclass, _method_id: jmethodID, _is_static: jboolean) -> jobject {
    unimplemented!()
}

extern "C" fn to_reflected_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _is_static: jboolean) -> jobject {
    unimplemented!()
}

extern "C" fn throw(_env: *mut JNIEnv, _obj: jthrowable) -> jint {
    unimplemented!()
}

extern "C" fn exception_occurred(_env: *mut JNIEnv) -> jthrowable {
    unimplemented!()
}

extern "C" fn exception_describe(_env: *mut JNIEnv) {
    unimplemented!()
}

extern "C" fn exception_clear(_env: *mut JNIEnv) {
    unimplemented!()
}

extern "C" fn fatal_error(_env: *mut JNIEnv, _msg: *const c_char) {
    unimplemented!()
}

extern "C" fn push_local_frame(_env: *mut JNIEnv, _capacity: jint) -> jint {
    unimplemented!()
}

extern "C" fn pop_local_frame(_env: *mut JNIEnv, _result: jobject) -> jobject {
    unimplemented!()
}

extern "C" fn new_global_ref(_env: *mut JNIEnv, _obj: jobject) -> jobject {
    unimplemented!()
}

extern "C" fn delete_global_ref(_env: *mut JNIEnv, _global_ref: jobject) {
    unimplemented!()
}

extern "C" fn delete_local_ref(_env: *mut JNIEnv, _local_ref: jobject) {
    unimplemented!()
}

extern "C" fn is_same_object(_env: *mut JNIEnv, _obj1: jobject, _obj2: jobject) -> jboolean {
    unimplemented!()
}

extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    unimplemented!()
}

extern "C" fn alloc_object(_env: *mut JNIEnv, _clazz: jclass) -> jobject {
    unimplemented!()
}

extern "C" fn get_object_class(_env: *mut JNIEnv, _obj: jobject) -> jclass {
    unimplemented!()
}

extern "C" fn is_instance_of(_env: *mut JNIEnv, _obj: jobject, _clazz: jclass) -> jboolean {
    unimplemented!()
}

extern "C" fn get_method_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jmethodID {
    unimplemented!()
}

extern "C" fn get_field_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jfieldID {
    unimplemented!()
}

extern "C" fn get_object_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jobject {
    unimplemented!()
}

extern "C" fn get_boolean_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jboolean {
    unimplemented!()
}

extern "C" fn get_byte_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jbyte {
    unimplemented!()
}

extern "C" fn get_char_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jchar {
    unimplemented!()
}

extern "C" fn get_short_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jshort {
    unimplemented!()
}

extern "C" fn get_int_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jint {
    unimplemented!()
}

extern "C" fn get_long_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jlong {
    unimplemented!()
}

extern "C" fn get_float_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jfloat {
    unimplemented!()
}

extern "C" fn get_double_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID) -> jdouble {
    unimplemented!()
}

extern "C" fn set_object_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jobject) {
    unimplemented!()
}

extern "C" fn set_boolean_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jboolean) {
    unimplemented!()
}

extern "C" fn set_byte_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jbyte) {
    unimplemented!()
}

extern "C" fn set_char_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jchar) {
    unimplemented!()
}

extern "C" fn set_short_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jshort) {
    unimplemented!()
}

extern "C" fn set_int_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jint) {
    unimplemented!()
}

extern "C" fn set_long_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jlong) {
    unimplemented!()
}

extern "C" fn set_float_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jfloat) {
    unimplemented!()
}

extern "C" fn set_double_field(_env: *mut JNIEnv, _obj: jobject, _field_id: jfieldID, _value: jdouble) {
    unimplemented!()
}

extern "C" fn monitor_enter(_env: *mut JNIEnv, _obj: jobject) -> jint {
    unimplemented!()
}

extern "C" fn monitor_exit(_env: *mut JNIEnv, _obj: jobject) -> jint {
    unimplemented!()
}

extern "C" fn get_array_length(_env: *mut JNIEnv, _array: jobject) -> jsize {
    unimplemented!()
}

extern "C" fn new_object_array(_env: *mut JNIEnv, _length: jsize, _clazz: jclass, _init: jobject) -> jobject {
    unimplemented!()
}

extern "C" fn get_object_array_element(_env: *mut JNIEnv, _array: jobject, _index: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn set_object_array_element(_env: *mut JNIEnv, _array: jobject, _index: jsize, _value: jobject) {
    unimplemented!()
}

extern "C" fn new_boolean_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_byte_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_char_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_short_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_int_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_long_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_float_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn new_double_array(_env: *mut JNIEnv, _length: jsize) -> jobject {
    unimplemented!()
}

extern "C" fn get_boolean_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jboolean {
    unimplemented!()
}

extern "C" fn get_byte_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jbyte {
    unimplemented!()
}

extern "C" fn get_char_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jchar {
    unimplemented!()
}

extern "C" fn get_short_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jshort {
    unimplemented!()
}

extern "C" fn get_int_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jint {
    unimplemented!()
}

extern "C" fn get_long_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jlong {
    unimplemented!()
}

extern "C" fn get_float_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jfloat {
    unimplemented!()
}

extern "C" fn get_double_array_elements(_env: *mut JNIEnv, _array: jobject, _is_copy: *mut jboolean) -> *mut jdouble {
    unimplemented!()
}

extern "C" fn release_boolean_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jboolean, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_byte_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jbyte, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_char_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jchar, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_short_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jshort, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_int_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jint, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_long_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jlong, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_float_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jfloat, _mode: jint) {
    unimplemented!()
}

extern "C" fn release_double_array_elements(_env: *mut JNIEnv, _array: jobject, _elems: *mut jdouble, _mode: jint) {
    unimplemented!()
}

extern "C" fn get_boolean_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jboolean) {
    unimplemented!()
}

extern "C" fn get_byte_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jbyte) {
    unimplemented!()
}

extern "C" fn get_char_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jchar) {
    unimplemented!()
}

extern "C" fn get_short_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jshort) {
    unimplemented!()
}

extern "C" fn get_int_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jint) {
    unimplemented!()
}

extern "C" fn get_long_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jlong) {
    unimplemented!()
}

extern "C" fn get_float_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jfloat) {
    unimplemented!()
}

extern "C" fn get_double_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *mut jdouble) {
    unimplemented!()
}

extern "C" fn set_boolean_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jboolean) {
    unimplemented!()
}

extern "C" fn set_byte_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jbyte) {
    unimplemented!()
}

extern "C" fn set_char_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jchar) {
    unimplemented!()
}

extern "C" fn set_short_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jshort) {
    unimplemented!()
}

extern "C" fn set_int_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jint) {
    unimplemented!()
}

extern "C" fn set_long_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jlong) {
    unimplemented!()
}

extern "C" fn set_float_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jfloat) {
    unimplemented!()
}

extern "C" fn set_double_array_region(_env: *mut JNIEnv, _array: jobject, _start: jsize, _len: jsize, _buf: *const jdouble) {
    unimplemented!()
}

// Add missing stubs for JNINativeInterface_
extern "C" fn throw_new(_env: *mut JNIEnv, _clazz: jclass, _msg: *const c_char) -> jint {
    unimplemented!()
}

extern "C" fn new_local_ref(_env: *mut JNIEnv, _obj: jobject) -> jobject {
    unimplemented!()
}

extern "C" fn get_static_method_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jmethodID {
    unimplemented!()
}

extern "C" fn get_static_field_id(_env: *mut JNIEnv, _clazz: jclass, _name: *const c_char, _sig: *const c_char) -> jfieldID {
    unimplemented!()
}

extern "C" fn get_static_object_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jobject { unimplemented!() }
extern "C" fn get_static_boolean_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jboolean { unimplemented!() }
extern "C" fn get_static_byte_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jbyte { unimplemented!() }
extern "C" fn get_static_char_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jchar { unimplemented!() }
extern "C" fn get_static_short_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jshort { unimplemented!() }
extern "C" fn get_static_int_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jint { unimplemented!() }
extern "C" fn get_static_long_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jlong { unimplemented!() }
extern "C" fn get_static_float_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jfloat { unimplemented!() }
extern "C" fn get_static_double_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID) -> jdouble { unimplemented!() }

extern "C" fn set_static_object_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jobject) { unimplemented!() }
extern "C" fn set_static_boolean_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jboolean) { unimplemented!() }
extern "C" fn set_static_byte_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jbyte) { unimplemented!() }
extern "C" fn set_static_char_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jchar) { unimplemented!() }
extern "C" fn set_static_short_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jshort) { unimplemented!() }
extern "C" fn set_static_int_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jint) { unimplemented!() }
extern "C" fn set_static_long_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jlong) { unimplemented!() }
extern "C" fn set_static_float_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jfloat) { unimplemented!() }
extern "C" fn set_static_double_field(_env: *mut JNIEnv, _clazz: jclass, _field_id: jfieldID, _value: jdouble) { unimplemented!() }

extern "C" fn new_string(_env: *mut JNIEnv, _unicode_chars: *const jchar, _len: jsize) -> jstring {
    unimplemented!()
}

extern "C" fn get_string_length(_env: *mut JNIEnv, _string: jstring) -> jsize {
    unimplemented!()
}

extern "C" fn get_string_chars(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const jchar {
    unimplemented!()
}

extern "C" fn release_string_chars(_env: *mut JNIEnv, _string: jstring, _chars: *const jchar) {
    unimplemented!()
}

extern "C" fn new_string_utf(_env: *mut JNIEnv, _bytes: *const c_char) -> jstring {
    unimplemented!()
}

extern "C" fn get_string_utf_length(_env: *mut JNIEnv, _string: jstring) -> jsize {
    unimplemented!()
}

extern "C" fn get_string_utf_chars(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const c_char {
    unimplemented!()
}

extern "C" fn release_string_utf_chars(_env: *mut JNIEnv, _string: jstring, _utf: *const c_char) {
    unimplemented!()
}

extern "C" fn register_natives(_env: *mut JNIEnv, _clazz: jclass, _methods: *const JNINativeMethod, _n_methods: jint) -> jint {
    unimplemented!()
}

extern "C" fn unregister_natives(_env: *mut JNIEnv, _clazz: jclass) -> jint {
    unimplemented!()
}

extern "C" fn get_java_vm(_env: *mut JNIEnv, _vm: *mut *mut JavaVM) -> jint {
    unimplemented!()
}

extern "C" fn get_string_region(_env: *mut JNIEnv, _str: jstring, _start: jsize, _len: jsize, _buf: *mut jchar) {
    unimplemented!()
}

extern "C" fn get_string_utf_region(_env: *mut JNIEnv, _str: jstring, _start: jsize, _len: jsize, _buf: *mut c_char) {
    unimplemented!()
}

extern "C" fn get_primitive_array_critical(_env: *mut JNIEnv, _array: jarray, _is_copy: *mut jboolean) -> *mut c_void {
    unimplemented!()
}

extern "C" fn release_primitive_array_critical(_env: *mut JNIEnv, _array: jarray, _carray: *mut c_void, _mode: jint) {
    unimplemented!()
}

extern "C" fn get_string_critical(_env: *mut JNIEnv, _string: jstring, _is_copy: *mut jboolean) -> *const c_char {
    unimplemented!()
}

extern "C" fn release_string_critical(_env: *mut JNIEnv, _string: jstring, _cstring: *const c_char) {
    unimplemented!()
}

extern "C" fn new_weak_global_ref(_env: *mut JNIEnv, _obj: jobject) -> jweak {
    unimplemented!()
}

extern "C" fn delete_weak_global_ref(_env: *mut JNIEnv, _obj: jweak) {
    unimplemented!()
}

extern "C" fn exception_check(_env: *mut JNIEnv) -> jboolean {
    unimplemented!()
}

extern "C" fn new_direct_byte_buffer(_env: *mut JNIEnv, _address: *mut c_void, _capacity: jlong) -> jobject {
    unimplemented!()
}

extern "C" fn get_direct_buffer_address(_env: *mut JNIEnv, _buf: jobject) -> *mut c_void {
    unimplemented!()
}

extern "C" fn get_direct_buffer_capacity(_env: *mut JNIEnv, _buf: jobject) -> jlong {
    unimplemented!()
}

extern "C" fn get_object_ref_type(_env: *mut JNIEnv, _obj: jobject) -> jobjectRefType {
    unimplemented!()
}

extern "C" fn get_module(_env: *mut JNIEnv, _clazz: jclass) -> jobject {
    unimplemented!()
}

extern "C" fn is_virtual_thread(_env: *mut JNIEnv, _obj: jobject) -> jboolean {
    unimplemented!()
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

// Replace the entire JNI_NATIVE_INTERFACE static structure
static mut JNI_NATIVE_INTERFACE: JNINativeInterface = unsafe { JNINativeInterface {
    reserved0: std::ptr::null_mut(),
    reserved1: std::ptr::null_mut(),
    reserved2: std::ptr::null_mut(),
    reserved3: std::ptr::null_mut(),

    GetVersion: get_version,
    DefineClass: define_class,
    FindClass: find_class,
    FromReflectedMethod: from_reflected_method,
    FromReflectedField: from_reflected_field,
    ToReflectedMethod: to_reflected_method,
    GetSuperclass: get_super_class,
    IsAssignableFrom: is_assignable_from,
    ToReflectedField: to_reflected_field,
    Throw: throw,
    ThrowNew: throw_new,
    ExceptionOccurred: exception_occurred,
    ExceptionDescribe: exception_describe,
    ExceptionClear: exception_clear,
    FatalError: fatal_error,
    PushLocalFrame: push_local_frame,
    PopLocalFrame: pop_local_frame,
    NewGlobalRef: new_global_ref,
    DeleteGlobalRef: delete_global_ref,
    DeleteLocalRef: delete_local_ref,
    IsSameObject: is_same_object,
    NewLocalRef: new_local_ref,
    EnsureLocalCapacity: ensure_local_capacity,
    AllocObject: alloc_object,
    NewObject: new_object, 
    NewObjectV: unimplemented!(), 
    NewObjectA: unimplemented!(), 
    GetObjectClass: get_object_class,
    IsInstanceOf: is_instance_of,
    GetMethodID: get_method_id,
    CallObjectMethod: call_object_method, 
    CallObjectMethodV: unimplemented!(),
    CallObjectMethodA: unimplemented!(),
    CallBooleanMethod: call_boolean_method, 
    CallBooleanMethodV: unimplemented!(),
    CallBooleanMethodA: unimplemented!(),
    CallByteMethod: call_byte_method, 
    CallByteMethodV: unimplemented!(),
    CallByteMethodA: unimplemented!(),
    CallCharMethod: call_char_method, 
    CallCharMethodV: unimplemented!(),
    CallCharMethodA: unimplemented!(),
    CallShortMethod: call_short_method, 
    CallShortMethodV: unimplemented!(),
    CallShortMethodA: unimplemented!(),
    CallIntMethod: call_int_method, 
    CallIntMethodV: unimplemented!(),
    CallIntMethodA: unimplemented!(),
    CallLongMethod: call_long_method, 
    CallLongMethodV: unimplemented!(),
    CallLongMethodA: unimplemented!(),
    CallFloatMethod: call_float_method, 
    CallFloatMethodV: unimplemented!(),
    CallFloatMethodA: unimplemented!(),
    CallDoubleMethod: call_double_method, 
    CallDoubleMethodV: unimplemented!(),
    CallDoubleMethodA: unimplemented!(),
    CallVoidMethod: call_void_method, 
    CallVoidMethodV: unimplemented!(),
    CallVoidMethodA: unimplemented!(),
    CallNonvirtualObjectMethod: call_nonvirtual_object_method, 
    CallNonvirtualObjectMethodV: unimplemented!(),
    CallNonvirtualObjectMethodA: unimplemented!(),
    CallNonvirtualBooleanMethod: call_nonvirtual_boolean_method, 
    CallNonvirtualBooleanMethodV: unimplemented!(),
    CallNonvirtualBooleanMethodA: unimplemented!(),
    CallNonvirtualByteMethod: call_nonvirtual_byte_method, 
    CallNonvirtualByteMethodV: unimplemented!(),
    CallNonvirtualByteMethodA: unimplemented!(),
    CallNonvirtualCharMethod: call_nonvirtual_char_method, 
    CallNonvirtualCharMethodV: unimplemented!(),
    CallNonvirtualCharMethodA: unimplemented!(),
    CallNonvirtualShortMethod: call_nonvirtual_short_method, 
    CallNonvirtualShortMethodV: unimplemented!(),
    CallNonvirtualShortMethodA: unimplemented!(),
    CallNonvirtualIntMethod: call_nonvirtual_int_method, 
    CallNonvirtualIntMethodV: unimplemented!(),
    CallNonvirtualIntMethodA: unimplemented!(),
    CallNonvirtualLongMethod: call_nonvirtual_long_method, 
    CallNonvirtualLongMethodV: unimplemented!(),
    CallNonvirtualLongMethodA: unimplemented!(),
    CallNonvirtualFloatMethod: call_nonvirtual_float_method, 
    CallNonvirtualFloatMethodV: unimplemented!(),
    CallNonvirtualFloatMethodA: unimplemented!(),
    CallNonvirtualDoubleMethod: call_nonvirtual_double_method, 
    CallNonvirtualDoubleMethodV: unimplemented!(),
    CallNonvirtualDoubleMethodA: unimplemented!(),
    CallNonvirtualVoidMethod: call_nonvirtual_void_method, 
    CallNonvirtualVoidMethodV: unimplemented!(),
    CallNonvirtualVoidMethodA: unimplemented!(),
    GetFieldID: get_field_id,
    GetObjectField: get_object_field,
    GetBooleanField: get_boolean_field,
    GetByteField: get_byte_field,
    GetCharField: get_char_field,
    GetShortField: get_short_field,
    GetIntField: get_int_field,
    GetLongField: get_long_field,
    GetFloatField: get_float_field,
    GetDoubleField: get_double_field,
    SetObjectField: set_object_field,
    SetBooleanField: set_boolean_field,
    SetByteField: set_byte_field,
    SetCharField: set_char_field,
    SetShortField: set_short_field,
    SetIntField: set_int_field,
    SetLongField: set_long_field,
    SetFloatField: set_float_field,
    SetDoubleField: set_double_field,
    GetStaticMethodID: get_static_method_id,
    CallStaticObjectMethod: call_static_object_method, 
    CallStaticObjectMethodV: unimplemented!(),
    CallStaticObjectMethodA: unimplemented!(),
    CallStaticBooleanMethod: call_static_boolean_method, 
    CallStaticBooleanMethodV: unimplemented!(),
    CallStaticBooleanMethodA: unimplemented!(),
    CallStaticByteMethod: call_static_byte_method, 
    CallStaticByteMethodV: unimplemented!(),
    CallStaticByteMethodA: unimplemented!(),
    CallStaticCharMethod: call_static_char_method, 
    CallStaticCharMethodV: unimplemented!(),
    CallStaticCharMethodA: unimplemented!(),
    CallStaticShortMethod: call_static_short_method, 
    CallStaticShortMethodV: unimplemented!(),
    CallStaticShortMethodA: unimplemented!(),
    CallStaticIntMethod: call_static_int_method, 
    CallStaticIntMethodV: unimplemented!(),
    CallStaticIntMethodA: unimplemented!(),
    CallStaticLongMethod: call_static_long_method, 
    CallStaticLongMethodV: unimplemented!(),
    CallStaticLongMethodA: unimplemented!(),
    CallStaticFloatMethod: call_static_float_method, 
    CallStaticFloatMethodV: unimplemented!(),
    CallStaticFloatMethodA: unimplemented!(),
    CallStaticDoubleMethod: call_static_double_method, 
    CallStaticDoubleMethodV: unimplemented!(),
    CallStaticDoubleMethodA: unimplemented!(),
    CallStaticVoidMethod: call_static_void_method, 
    CallStaticVoidMethodV: unimplemented!(),
    CallStaticVoidMethodA: unimplemented!(),
    GetStaticFieldID: get_static_field_id,
    GetStaticObjectField: get_static_object_field,
    GetStaticBooleanField: get_static_boolean_field,
    GetStaticByteField: get_static_byte_field,
    GetStaticCharField: get_static_char_field,
    GetStaticShortField: get_static_short_field,
    GetStaticIntField: get_static_int_field,
    GetStaticLongField: get_static_long_field,
    GetStaticFloatField: get_static_float_field,
    GetStaticDoubleField: get_static_double_field,
    SetStaticObjectField: set_static_object_field,
    SetStaticBooleanField: set_static_boolean_field,
    SetStaticByteField: set_static_byte_field,
    SetStaticCharField: set_static_char_field,
    SetStaticShortField: set_static_short_field,
    SetStaticIntField: set_static_int_field,
    SetStaticLongField: set_static_long_field,
    SetStaticFloatField: set_static_float_field,
    SetStaticDoubleField: set_static_double_field,
    NewString: new_string,
    GetStringLength: get_string_length,
    GetStringChars: get_string_chars,
    ReleaseStringChars: release_string_chars,
    NewStringUTF: new_string_utf,
    GetStringUTFLength: get_string_utf_length,
    GetStringUTFChars: get_string_utf_chars,
    ReleaseStringUTFChars: release_string_utf_chars,
    GetArrayLength: get_array_length,
    NewObjectArray: new_object_array,
    GetObjectArrayElement: get_object_array_element,
    SetObjectArrayElement: set_object_array_element,
    NewBooleanArray: new_boolean_array,
    NewByteArray: new_byte_array,
    NewCharArray: new_char_array,
    NewShortArray: new_short_array,
    NewIntArray: new_int_array,
    NewLongArray: new_long_array,
    NewFloatArray: new_float_array,
    NewDoubleArray: new_double_array,
    GetBooleanArrayElements: get_boolean_array_elements,
    GetByteArrayElements: get_byte_array_elements,
    GetCharArrayElements: get_char_array_elements,
    GetShortArrayElements: get_short_array_elements,
    GetIntArrayElements: get_int_array_elements,
    GetLongArrayElements: get_long_array_elements,
    GetFloatArrayElements: get_float_array_elements,
    GetDoubleArrayElements: get_double_array_elements,
    ReleaseBooleanArrayElements: release_boolean_array_elements,
    ReleaseByteArrayElements: release_byte_array_elements,
    ReleaseCharArrayElements: release_char_array_elements,
    ReleaseShortArrayElements: release_short_array_elements,
    ReleaseIntArrayElements: release_int_array_elements,
    ReleaseLongArrayElements: release_long_array_elements,
    ReleaseFloatArrayElements: release_float_array_elements,
    ReleaseDoubleArrayElements: release_double_array_elements,
    GetBooleanArrayRegion: get_boolean_array_region,
    GetByteArrayRegion: get_byte_array_region,
    GetCharArrayRegion: get_char_array_region,
    GetShortArrayRegion: get_short_array_region,
    GetIntArrayRegion: get_int_array_region,
    GetLongArrayRegion: get_long_array_region,
    GetFloatArrayRegion: get_float_array_region,
    GetDoubleArrayRegion: get_double_array_region,
    SetBooleanArrayRegion: set_boolean_array_region,
    SetByteArrayRegion: set_byte_array_region,
    SetCharArrayRegion: set_char_array_region,
    SetShortArrayRegion: set_short_array_region,
    SetIntArrayRegion: set_int_array_region,
    SetLongArrayRegion: set_long_array_region,
    SetFloatArrayRegion: set_float_array_region,
    SetDoubleArrayRegion: set_double_array_region,
    RegisterNatives: register_natives,
    UnregisterNatives: unregister_natives,
    MonitorEnter: monitor_enter,
    MonitorExit: monitor_exit,
    GetJavaVM: get_java_vm,
    GetStringRegion: get_string_region,
    GetStringUTFRegion: get_string_utf_region,
    GetPrimitiveArrayCritical: get_primitive_array_critical,
    ReleasePrimitiveArrayCritical: release_primitive_array_critical,
    GetStringCritical: get_string_critical,
    ReleaseStringCritical: release_string_critical,
    NewWeakGlobalRef: new_weak_global_ref,
    DeleteWeakGlobalRef: delete_weak_global_ref,
    ExceptionCheck: exception_check,
    NewDirectByteBuffer: new_direct_byte_buffer,
    GetDirectBufferAddress: get_direct_buffer_address,
    GetDirectBufferCapacity: get_direct_buffer_capacity,
    GetObjectRefType: get_object_ref_type,
    GetModule: get_module,
    IsVirtualThread: is_virtual_thread,
}};


