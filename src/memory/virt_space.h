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

#ifndef MEMORY_VIRT_SPACE_H_
#define MEMORY_VIRT_SPACE_H_

#include "utils/global_defs.h"

typedef struct VirtSpace VirtSpace;
struct VirtSpace {
  byte_t* start;
  size_t reserved;
  size_t committed;

  size_t _alignment;
  
  bool _executable;
};

bool VirtSpace_init(VirtSpace*, byte_t* start, size_t byte_size, size_t alignment, bool executable);
void VirtSpace_dtor(VirtSpace*);

bool VirtSpace_contains(VirtSpace*, void*);

bool VirtSpace_expand_by(VirtSpace*, size_t byte_size, bool pretouch);
bool VirtSpace_shrink_by(VirtSpace*, size_t byte_size);

#endif // MEMORY_VIRT_SPACE_H_


