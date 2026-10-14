#ifndef CORE_UTILS_ALIGN_H_
#define CORE_UTILS_ALIGN_H_

#include <stdbool.h>
#include <stddef.h>

/* Alignment must be nonzero. */
static inline bool is_aligned(size_t n, size_t alignment) {
    return n % alignment == 0;
}

/* Alignment must be a nonzero power of two. */
static inline size_t align_down(size_t n, size_t alignment) {
    return n & ~(alignment - 1);
}

/* Alignment must be a nonzero power of two, and n + alignment - 1
 * must fit in size_t. Callers must check rounding overflow first. */
static inline size_t align_up(size_t n, size_t alignment) {
    return align_down(n + alignment - 1, alignment);
}

#endif /* CORE_UTILS_ALIGN_H_ */
