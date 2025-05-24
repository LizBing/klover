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

use std::{ffi::{c_char, c_void}, ptr::{null, null_mut}};

use crate::prims::jni::JNI_ERR;

use super::jni::{jbyte, jclass, jint, jobject, jsize, JNIEnv, JavaVM, JNI_OK, JNI_VERSION};

enum VMCreationState {
    NotCreated,
    InProgress,
    Complete
}

static mut VM_CREATED: VMCreationState = VMCreationState::NotCreated;
static mut MAIN_VM: JavaVM = null_mut();

#[no_mangle]
extern "C" fn JNI_GetDefaultJavaVMInitArgs(args: *mut c_void) -> jint {
    JNI_ERR
}

#[no_mangle]
extern "C" fn JNI_CreateJavaVM(pvm: *mut *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint {
    println!("Hello!");

    JNI_ERR
}

#[no_mangle]
unsafe extern "C" fn JNI_GetCreatedJavaVMs(vm_buf: *mut *mut JavaVM, len: jsize, nvms: *mut jsize) -> jint {
    if let VMCreationState::Complete = VM_CREATED {
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

// Methods in JNINativeInterface_

fn get_version(env: *mut JNIEnv) -> jint {
    JNI_VERSION
}

fn define_class(env: *mut JNIEnv, name: *const c_char, loader: jobject, buf: *const jbyte, len: jsize) -> jclass {
    unimplemented!()
}
