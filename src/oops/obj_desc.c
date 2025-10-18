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

#include "oops/mark_word.h"
#include "utils/global_defs.h"

typedef struct ObjDesc ObjDesc;
struct ObjDesc {
  MarkWord _mark_word;
  byte_t _data[0];
};

typedef struct ArrayObjDesc ArrayObjDesc;
struct ArrayObjDesc {
  ObjDesc _super;
  uint32_t _len;

  LP64_ONLY(uint32_t _;)
  byte_t _data[0];
};
