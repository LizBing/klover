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

#ifndef UTIL_LLIST_
#define UTIL_LLIST_

#include "util/globalDefinitions.h"

typedef struct LListNode {
    struct LListNode* prev;
    struct LListNode* next;
} LListNode, LList;

static inline void LListNode_insert(LListNode* pos, LListNode* n) {
    LListNode* next = pos->next;

    n->next = next;
    n->prev = pos;

    pos->next = n;
    next->prev = n;
}

static inline void LListNode_erase(LListNode* pos) {
    LListNode* next = pos->next;
    LListNode* prev = pos->prev;

    next->prev = prev;
    prev->next = next;
}

static inline void LList_init(LList* this) {
    this->next = this;
    this->prev = this;
}

static inline LListNode* LList_begin(LList* this) {
    return this->next;
}

static inline LListNode* LList_end(LList* this) {
    return this;
}

static inline LListNode* LList_last(LList* this) {
    return this->prev;
}

static inline void LList_pushBack(LList* this, LListNode* n) {
    LListNode_insert(this->prev, n);
}

static inline void LList_pushFront(LList* this, LListNode* n) {
    LListNode_insert(this, n);
}

static inline void LList_popFront(LList* this) {
    LListNode_erase(this->next);
}

static inline void LList_popBack(LList* this) {
    LListNode_erase(this->prev);
}

static inline bool LList_empty(LList* this) {
    return LList_begin(this) == LList_end(this);
}

static inline bool LList_iterate(LList* this, bool(*func)(LListNode*, void*), void* arg) {
    for (LListNode* iter = LList_begin(this); iter != LList_end(this); iter = iter->next) 
        if (!func(iter, arg))
            return false;
    return true;
}

static inline bool LList_doReservedIteration(
    LList* this, bool(*func)(LListNode*, void*), void* arg
) {
    for (LListNode* iter = LList_last(this); iter != LList_end(this); iter = iter->prev)
        if (!func(iter, arg))
            return false;

    return true;
}

#endif
