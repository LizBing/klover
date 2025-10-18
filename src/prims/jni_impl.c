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

#ifndef _JNI_IMPLEMENTATION_
#define _JNI_IMPLEMENTATION_

#include "prims/jni.h"

static JavaVMInitArgs DEFAULT_INIT_ARGS = {
  .version = JNI_VERSION_21,
  .nOptions = 0,
  .options = NULL,
  .ignoreUnrecognized = 1
};

jint JNI_GetDefaultJavaVMInitArgs(void* args) {
  *(JavaVMInitArgs*)args = DEFAULT_INIT_ARGS;

  return JNI_OK;
}

jint JNI_CreateJavaVM(JavaVM** pvm, void** penv, void* args) {}

#endif /* _JNI_IMPLEMENTATION_ */
