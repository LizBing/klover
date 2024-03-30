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
#include "runtime/mutex.h"
#include "util/globalDefinitions.h"

Mutex* new_Mutex() {
    Mutex* m = CHeap_alloc(sizeof(Mutex));
    pthread_mutex_init(m, NULL);

    return m;
}

void delete_Mutex(Mutex* this) {
    pthread_mutex_destroy(this);
    CHeap_free(this);
}

inline void Mutex_lock(Mutex* this) { pthread_mutex_lock(this); }
inline void Mutex_unlock(Mutex* this) { pthread_mutex_unlock(this); }
inline bool Mutex_try_lock(Mutex* this) { return pthread_mutex_trylock(this); }
