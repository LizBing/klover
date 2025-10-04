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

#include "memory/allocation.h"
#include "memory/box.h"
#include "utils/global_defs.h"

typedef struct ContextImpl ContextImpl;
struct ContextImpl {};

ContextImpl* ContextImpl_new(size_t stack_byte_size) {
  ContextImpl res = {};
  box_and_return(&res, true);
}

void ContextImpl_delete(ContextImpl* this) {
  c_heap_free(this);
}
