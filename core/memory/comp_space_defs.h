#ifndef MEMORY_COMP_SPACE_DEFS_H_
#define MEMORY_COMP_SPACE_DEFS_H_

#include "utils/global_defs.h"

static const uintptr_t METASPACE_BASE = 1ul << 43;
static const uintptr_t GCHEAP_BASE = 1ul << 44;
static const size_t COMPSPACE_BYTE_SIZE = 32ul * G;

/*
 * With 8-byte alignment (sizeof(HeapWord) == 8) and a 32-bit compressed
 * pointer, the addressable range is 2^32 * 2^3 = 32 GB, which matches
 * COMPSPACE_BYTE_SIZE exactly.
 *
 * Zero-sentinel null preservation:
 *   - Compressed value 0 is reserved for NULL.
 *   - Effective narrow range: [1 .. 2^32-1], still covering the full 32 GB
 *     because address 'base' itself (shifted offset 0) is never a valid
 *     object location (allocations start above base).
 *
 *   encode:  narrow = 0                     when ptr == NULL
 *            narrow = (ptr - base) >> 3      when ptr != NULL
 *   decode:  ptr = NULL                      when narrow == 0
 *            ptr = base + (narrow << 3)      when narrow != 0
 */
static const size_t COMP_PTR_SHIFT = 3;

typedef uint32_t comp_ptr_t;

#define comp_ptr_encode(base, ptr)                                      \
    ((ptr) == NULL                                                       \
         ? (comp_ptr_t)0                                                  \
         : (comp_ptr_t)(((uintptr_t)(ptr) - (uintptr_t)(base)) >> COMP_PTR_SHIFT))

#define comp_ptr_decode(base, comp_ptr)                                 \
    ((comp_ptr) == 0                                                      \
         ? NULL                                                           \
         : (HeapWord*)((uintptr_t)(base) + ((uintptr_t)(comp_ptr) << COMP_PTR_SHIFT)))

#endif /* MEMORY_COMP_SPACE_DEFS_H_ */
