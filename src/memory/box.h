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

#ifndef MEMORY_BOX_H_
#define MEMORY_BOX_H_

#include <string.h>

#include "memory/allocation.h"

static inline void* _box_and_return(void* src, size_t byte_size, bool oom_if_failed) {
  void* res = c_heap_alloc(byte_size, oom_if_failed);
  if (NULL != res) {
    memcpy(res, src, byte_size);
  }

  return res;
}

#define box_and_return(ptr, oom_if_failed) \
  return _box_and_return(ptr, sizeof(*(ptr)), oom_if_failed)


#endif // MEMORY_BOX_H_

