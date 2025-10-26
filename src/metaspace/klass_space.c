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

#include "memory/virt_space.h"
#include "runtime/mutex_locker.h"
#include "utils/global_defs.h"

static VirtSpace VIRT_SPACE = { 0 };

void KlassSpace_initialize(size_t _log_slot_byte_size) { unimplemented(); }

void* KlassSpace_allocate() { unimplemented(); }

void* KlassSpace_base() { unimplemented(); }

/*
int KlassSpace_encode(void* raw) { unimplemented(); }

void* KlassSpace_decode(int narrow) { unimplemented(); }
*/

// Class unloading is not supported currently.
void KlassSpace_free(void* ptr) { ; }
