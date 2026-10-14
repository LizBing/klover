#ifndef CORE_UTILS_ASSERT_H_
#define CORE_UTILS_ASSERT_H_

#include <stdio.h>
#include <stdlib.h>

#define PANIC(...) do {\
  fprintf(stderr, __VA_ARGS__);\
  exit(-1);\
} while (0)

#define ASSERT(expr, ...) do {\
  if (!(expr)) {\
    PANIC(__VA_ARGS__);\
  }\
} while (0)

#endif /* CORE_UTILS_ASSERT_H_ */

