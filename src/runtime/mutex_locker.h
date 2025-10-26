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

#ifndef RUNTIME_MUTEX_LOCKER_H_
#define RUNTIME_MUTEX_LOCKER_H_

#include "runtime/mutex.h"
#include "utils/global_defs.h"

extern Mutex* KLASS_SPACE_LOCK;

#define mutex_locker(mtx, block) \
{\
  assert(Mutex_lock(mtx), "bad lock");\
  block\
  assert(Mutex_unlock(mtx), "bad lock");\
}

#define mutex_unlocker(mtx, block) \
{\
  assert(Mutex_unlock(mtx), "bad lock");\
  block\
  assert(Mutex_lock(mtx), "bad lock");\
}

#endif // RUNTIME_MUTEX_LOCKER_H_
