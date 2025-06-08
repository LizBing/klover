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

#define _JNI_IMPLEMENTATION_
#include "jni.h"

#define DUMMY_OBJ 1

jobject new_object(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jobject)DUMMY_OBJ;
}

jobject call_object_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jobject)DUMMY_OBJ;
}

jboolean call_boolean_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jboolean)0;
}

jbyte call_byte_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jbyte)0;
}

jchar call_char_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jchar)0;
}

jshort call_short_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jshort)0;
}

jint call_int_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jint)0;
}

jlong call_long_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jlong)0;
}

jfloat call_float_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jfloat)0.0f;
}

jdouble call_double_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    return (jdouble)0.0;
}

void call_void_method(JNIEnv *env, jobject obj, jmethodID method_id, ...) {
    // Dummy implementation does nothing
}

jobject call_nonvirtual_object_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jobject)DUMMY_OBJ;
}

jboolean call_nonvirtual_boolean_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jboolean)0;
}

jbyte call_nonvirtual_byte_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jbyte)0;
}

jchar call_nonvirtual_char_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jchar)0;
}

jshort call_nonvirtual_short_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jshort)0;
}

jint call_nonvirtual_int_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jint)0;
}

jlong call_nonvirtual_long_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jlong)0;
}

jfloat call_nonvirtual_float_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jfloat)0.0f;
}

jdouble call_nonvirtual_double_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    return (jdouble)0.0;
}

void call_nonvirtual_void_method(JNIEnv *env, jobject obj, jclass clazz, jmethodID method_id, ...) {
    // Dummy implementation does nothing
}

jobject call_static_object_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jobject)DUMMY_OBJ;
}

jboolean call_static_boolean_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jboolean)0;
}

jbyte call_static_byte_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jbyte)0;
}

jchar call_static_char_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jchar)0;
}

jshort call_static_short_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jshort)0;
}

jint call_static_int_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jint)0;
}

jlong call_static_long_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jlong)0;
}

jfloat call_static_float_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jfloat)0.0f;
}

jdouble call_static_double_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    return (jdouble)0.0;
}

void call_static_void_method(JNIEnv *env, jclass clazz, jmethodID method_id, ...) {
    // Dummy implementation does nothing
}



