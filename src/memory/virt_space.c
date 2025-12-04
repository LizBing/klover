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
#include "memory/virt_space.h"
#include "memory/box.h"
#include "runtime/os.h"
#include "utils/align.h"

VirtSpace* VirtSpace_new(byte_t* start, size_t byte_size, size_t alignment, bool executable) {
  alignment = align_up(alignment, os_vm_page_size());
  byte_size = align_up(byte_size, alignment);

  start = os_reserve_memory(start, byte_size);
  if (NULL == start) {
    return false;
  }

  VirtSpace space = (VirtSpace) {
    .start = start,
    .reserved = byte_size,
    .committed = 0,

    ._alignment = alignment,

    ._executable = executable
  };

  box_and_return(&space);
}

void VirtSpace_delete(VirtSpace* this) {
  os_release_memory(this->start, this->reserved);
  c_heap_free(this);
}

bool VirtSpace_contains(VirtSpace* this, void* addr) {
  return (void*)this->start <= addr && addr < (void*)(this->start + this->reserved);
}

static byte_t* current_top(VirtSpace* this) {
  return this->start + this->committed;
}

bool VirtSpace_expand_by(VirtSpace* this, size_t byte_size, bool pretouch) {
  byte_size = align_up(byte_size, this->_alignment);

  byte_t* new_top = current_top(this) + byte_size;
  if (!VirtSpace_contains(this, new_top)) {
    return false;
  }

  if (os_commit_memory(current_top(this), byte_size, this->_executable)) {
    if (pretouch) {
      os_pretouch_memory(current_top(this), byte_size);
      this->committed += byte_size;
    }
  } else {
    return false;
  }
  
  return true;;
}

bool VirtSpace_shrink_by(VirtSpace* this, size_t byte_size) {
  byte_size = align_down(byte_size, this->_alignment);
  if (0 == byte_size) {
    return true;
  }

  if (byte_size > this->committed) {
    return false;
  }

  if (os_uncommit_memory(current_top(this) - byte_size, byte_size)) {
    this->committed -= byte_size;
  } else {
    return false;
  }

  return true;
}

