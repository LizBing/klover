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

#ifndef RUNTIME_OS_H_
#define RUNTIME_OS_H_

#include "utils/global_defs.h"

size_t os_vm_page_size();

byte_t* os_reserve_memory(byte_t*, size_t byte_size);
bool os_commit_memory(byte_t*, size_t byte_size, bool executable);
bool os_uncommit_memory(byte_t*, size_t byte_size);
bool os_release_memory(byte_t*, size_t byte_size);

void os_pretouch_memory(byte_t*, size_t byte_size);

#endif // RUNTIME_OS_H_

