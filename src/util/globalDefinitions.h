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

#ifndef UTIL_GLOBALDEFINITIONS_H
#define UTIL_GLOBALDEFINITIONS_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "util/macros.h"

typedef const char* str;
typedef int8_t byte;

typedef intptr_t intx;
typedef uintptr_t uintx;

#define MAX2(x, y) (x > y ? x : y)
#define MIN2(x, y) (x < y ? x : y)

#define oneBit ((uintx)1)
#define nth_bit(n) (oneBit << (n))
#define right_nBits(s) (nth_bit(s) - 1)

inline void setBits(uintx* n, uintx m) { *n |= m; }
inline void resetBits(uintx* n, uintx m) { *n &= ~m; }
inline uintx maskBits(uintx n, uintx m) { return n & m; }

inline bool areMaskBitsSet(uintx n, uintx m) { return maskBits(m, n) == m; }
inline bool isNthBitSet(uintx x, int n) { return x & nth_bit(n); }

static const size_t K = 1024;
static const size_t M = K * 1024;
static const size_t G = M * 1024;

static const int logBytesPerShort = 1;
static const int logBytesPerInt = 2;
static const int logBytesPerVMLong = 3;

static const int logBytesPerWord = LP64_ONLY(3) NOT_LP64(2);

static const int bytesPerShort = 1 << logBytesPerShort;
static const int bytesPerInt = 1 << logBytesPerInt;
static const int bytesPerVMLong = 1 << logBytesPerVMLong;
static const int bytesPerWord = 1 << logBytesPerWord;

static const int logBitsPerByte = 3;
static const int logBitsPerShort = logBitsPerByte + logBytesPerShort;
static const int logBitsPerInt = logBitsPerByte + logBytesPerInt;
static const int logBitsPerVMLong = logBitsPerByte + logBytesPerVMLong;
static const int logBitsPerWord = logBitsPerByte + logBytesPerWord;

static const int bitsPerByte = 1 << logBitsPerByte;
static const int bitsPerShort = 1 << logBitsPerShort;
static const int bitsPerInt = 1 << logBitsPerInt;
static const int bitsPerVMLong = 1 << logBitsPerVMLong;
static const int bitsPerWord = 1 << logBitsPerWord;

// Linux-style generic programming
#define containerof(ptr, type, member) \
    ((type*)((intptr_t)ptr - (intptr_t)&((type*)0)->member))

#define assert(expr, ...) do {\
    if (!(expr)) perror(__VA_ARGS__);\
} while(false)


#endif // UTIL_GLOBALDEFINITIONS_H

