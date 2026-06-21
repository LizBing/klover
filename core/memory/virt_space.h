#ifndef MEMORY_VIRT_SPACE_H_
#define MEMORY_VIRT_SPACE_H_

#include "utils/global_defs.h"

typedef struct VirtSpace VirtSpace;
struct VirtSpace {
    HeapWord* start;
    HeapWord* end;
    
    HeapWord* commit_top;
    bool exec;
};

// If base == 0, then let the system decide the address.
VirtSpace* create_virt_space(uintptr_t base, size_t word_size, bool exec);
void destroy_virt_space(VirtSpace*);

// Word size.
static inline size_t vs_reserved(VirtSpace* vs) {
    return (size_t)(vs->end - vs->start);
}

// Word size.
static inline size_t vs_committed(VirtSpace* vs) {
    return (size_t)(vs->commit_top - vs->start);
}

bool vs_expand(VirtSpace*, size_t word_size, bool touch);
bool vs_shrink(VirtSpace*, size_t word_size);

#endif /* CORE_MEMORY_VIRT_SPACE_H_ */
