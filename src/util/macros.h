/*
 * Copyright (c) 2024, Lei Zaakjyu. All rights reserved.
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

#ifndef UTIL_MACROS_H
#define UTIL_MACROS_H

#define STR(x) #x
#define XSTR(x) STR(x)

#define PASTE_TOKENS_AUX(x, y) x ## y
#define PASTE_TOKENS(x, y) PASTE_TOKENS_AUX(x, y)

#ifdef _LP64
#define LP64_ONLY(x) x
#define NOT_LP64(x)
#else
#define LP64_ONLY(x)
#define NOT_LP64(x) x
#endif

#define OS_HEADER_STEM(basename) PASTE_TOKENS(basename, INCLUDE_SUFFIX_OS)
#define OS_HEADER(basename) XSTR(OS_HEADER_STEM(basename).h)

#endif // UTIL_MACROS_H

