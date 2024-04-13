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
#include "runtime/threadCritical.h"
#include "util/align.h"
#include "util/llist.h"

static const int ARENA_ALIGNMENT = bytesPerWord;

typedef struct Chunk Chunk;
struct Chunk {
    size_t _len;

    Chunk* next;
    byte begin[0];
};

Chunk* new_Chunk(size_t len) {
    Chunk* this = CHeap_alloc(sizeof(Chunk) + len); 
    this->_len = len;

    return this;
}

inline size_t Chunk_length(Chunk* this) { return this->_len; }


typedef struct {
    Chunk* _top;
    size_t _size;   // size of a single managed chunk
} ChunkPool;

void ChunkPool_push(ChunkPool* this, Chunk* n) {
    assert(this->_size == Chunk_length(n), "wrong pool");

    ThreadCritical_begin();

    n->next = this->_top;
    this->_top = n;

    ThreadCritical_end();
}

Chunk* ChunkPool_pop(ChunkPool* this) {
    ThreadCritical_begin();

    Chunk* n = this->_top;
    if (n != NULL) 
        this->_top = n->next;

    ThreadCritical_end();

    return n;
}

void ChunkPool_clear(ChunkPool* this) {
    for (Chunk* iter = this->_top; iter != NULL; iter = iter->next)
        CHeap_free(iter);
    this->_top = NULL;
}

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


void Chunk_chop(Chunk* this) {
    Chunk* iter = this;
    while (iter != NULL) {
        Chunk* n = iter;
        iter = n->next;

        ChunkPool* p = get_pool_for_size(this->_len);        
        if (p != NULL)
            ChunkPool_push(p, n);
        else CHeap_free(n);
    }
}

inline void Chunk_next_chop(Chunk* this) {
    Chunk_chop(this->next);
}

Chunk* allocate_chunk(size_t length) {
    assert(is_aligned(length, ARENA_ALIGNMENT), "should be aligned");

    ChunkPool* p = get_pool_for_size(length);
    Chunk* c = NULL;
    if (p != NULL)
        c = ChunkPool_pop(p);
    if (c == NULL)
        c = new_Chunk(length);

    return c;
}

struct Arena {
    Chunk* _top;

    byte* _begin;
    byte* _end;
};

Arena* new_Arena(size_t init_size) {
    init_size = align_up(init_size, ARENA_ALIGNMENT);

    Arena* this = CHeap_alloc(sizeof(Arena));

    Chunk* n = allocate_chunk(init_size);
    this->_top = n;

    this->_begin = n->begin;
    this->_end = n->begin + n->_len;
}

void delete_Arena(Arena* this) {
    Chunk_chop(this->_top);
    CHeap_free(this);
}


