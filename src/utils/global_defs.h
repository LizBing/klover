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

#ifndef UTILS_GLOBAL_DEFS_H_
#define UTILS_GLOBAL_DEFS_H_

#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>

typedef char byte_t;

#define assert(e, ...) \
do {\
  if (!(e)) {\
    printf("Assert: '" #e "' at File '%s', Line %d\n\t", __FILE__, __LINE__);\
    printf(__VA_ARGS__);\
    printf("\n");\
    exit(1);\
  }\
} while (0)

#define panic(...) \
do {\
  printf("Panicked at File '%s', Line %d\n\t", __FILE__, __LINE__);\
  printf(__VA_ARGS__);\
  printf("\n");\
  exit(1);\
} while (0)\

#ifdef _LP64
#define LP64_ONLY(x) x
#define NOT_LP64(x)
#else
#define LP64_ONLY(x)
#define NOT_LP64(x) x
#endif /* _LP64 */

#ifdef _LP32
#define LP32_ONLY(x) x
#define NOT_LP32(x)
#else
#define LP32_ONLY(x)
#define NOT_LP32(x) x
#endif /* _LP64 */

#endif // UTILS_GLOBAL_DEFS_H_

