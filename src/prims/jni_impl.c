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

#include <stdatomic.h>
#include <stdbool.h>

#include "./jni.h"

// stubs in JavaVM

jint JavaVM_DestroyJavaVM(JavaVM* vm) {
  return JNI_ERR;
}

jint JavaVM_AttachCurrentThread(JavaVM* vm, void** penv, void* args) {
  return JNI_ERR;
}

jint JavaVM_DetachCurrentThread(JavaVM* vm) {
  return JNI_ERR;
}

jint JavaVM_GetEnv(JavaVM* vm, void** penv, jint version) {
  return JNI_ERR;
}

jint JavaVM_AttachCurrentThreadAsDaemon(JavaVM* vm, void** penv, void* args) {
  return JNI_ERR;
}

static const struct JNIInvokeInterface_ JAVA_VM = {
  .reserved0 = NULL,
  .reserved1 = NULL,
  .reserved2 = NULL,

  .DestroyJavaVM = JavaVM_DestroyJavaVM,
  .AttachCurrentThread = JavaVM_AttachCurrentThread,
  .DetachCurrentThread = JavaVM_DetachCurrentThread,
  .GetEnv = JavaVM_GetEnv,
  .AttachCurrentThreadAsDaemon = JavaVM_AttachCurrentThreadAsDaemon,
};

static atomic_bool JVM_CREATED = false;

static JavaVMInitArgs DEFAULT_INIT_ARGS = {
  .version = JNI_VERSION_21,
  .nOptions = 0,
  .options = NULL,
  .ignoreUnrecognized = 1
};

static struct JNINativeInterface_ JNI_ENV;

jint JNI_GetDefaultJavaVMInitArgs(void* args) {
  *(JavaVMInitArgs*)args = DEFAULT_INIT_ARGS;

  return JNI_OK;
}

jint JNI_CreateJavaVM(JavaVM** pvm, void** penv, void* args) {
  return JNI_ERR;
}

jint JNI_GetCreatedJavaVMs(JavaVM** vmBuf, jsize bufLen, jsize* nVMs) {
  return JNI_ERR;
}

jint JNI_OnLoad(JavaVM* vm, void* reserved) {
  return JNI_ERR;
}

void JNI_OnUnload(JavaVM* vm, void* reserved);

jint JNI_GetVersion(JNIEnv* _env) {
  return JNI_VERSION_21;
}

static struct JNINativeInterface_ JNI_ENV = {
  .reserved0 = NULL,
  .reserved1 = NULL,
  .reserved2 = NULL,
  .reserved3 = NULL,

  .GetVersion = JNI_GetVersion,
};

