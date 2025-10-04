/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef MEMORY_ALLOCATION_H_
#define MEMORY_ALLOCATION_H_

#include "utils/global_defs.h"

// C Heap Allocation
void* c_heap_alloc(size_t byte_size, bool oom_if_failed);
void c_heap_free(void*);

#endif // MEMORY_ALLOCATION_H_

