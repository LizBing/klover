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

#include "memory/allocation.h"
#include "util/llist.h"

typedef struct Chunk Chunk;
struct Chunk {
    size_t _len;

    Chunk* next;
    byte start[0];
};

Chunk* new_Chunk(size_t len) {
    Chunk* this = CHeap_alloc(sizeof(Chunk) + len); 
    this->_len = len;

    return this;
}

void delete_Chunk(Chunk* this) { CHeap_free(this); }

inline size_t Chunk_length(Chunk* this) { return this->_len; }

typedef struct {
    Chunk* _first;
    int _chunks;

    size_t _size;   // size of a single managed chunk
} ChunkPool;

static const int _num_pools = 4;
static ChunkPool _pools[_num_pools] = {
    { ._size = ArenaChunk_tiny_size },
    { ._size = ArenaChunk_init_size },
    { ._size = ArenaChunk_medium_size },
    { ._size = ArenaChunk_size }
};

ChunkPool* get_pool_for_size(size_t s) {
    for (int i = 0; i < _num_pools; ++i) {
        ChunkPool* p = _pools + i;
        if (p->_size == s) return p;
    }

    return NULL;
}

struct Arena {
    ;
};
