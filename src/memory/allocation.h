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

#ifndef MEMORY_ALLOCATION_H_
#define MEMORY_ALLOCATION_H_

#include "util/globalDefinitions.h"

void* CHeap_alloc(size_t);  // zapped
void CHeap_free(void*);

// a hotspot-style Arena
typedef struct Arena Arena;
typedef struct ArenaMark ArenaMark;

typedef enum {
  ArenaChunk_slack = LP64_ONLY(40) NOT_LP64(24),

  ArenaChunk_tiny_size = 256 - ArenaChunk_slack,
  ArenaChunk_init_size = 1 * K - ArenaChunk_slack,
  ArenaChunk_medium_size = 10 * K - ArenaChunk_slack,
  ArenaChunk_size = 32 * K - ArenaChunk_slack,
  ArenaChunk_non_pool = ArenaChunk_init_size + 32
} ArenaChunkSize;

Arena* new_Arena(size_t init_size);
void delete_Arena(Arena*);

void* Arena_alloc(Arena*, size_t);
void Arena_try_free(Arena*, void*, size_t);
void* Arena_realloc(Arena*, void*, size_t old_size, size_t new_size);

ArenaMark* new_ArenaMark();
void delete_ArenaMark(ArenaMark*);

void Arena_mark(Arena*, ArenaMark*);
void Arena_restore(Arena*, ArenaMark*);

#endif // MEMORY_ALLOCATION_H_
