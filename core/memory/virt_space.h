#ifndef MEMORY_VIRT_SPACE_H_
#define MEMORY_VIRT_SPACE_H_

#include "utils/assert.h"
#include "utils/global_defs.h"
#include <unistd.h>

static inline
size_t get_vm_page_size() {
    long ps = sysconf(_SC_PAGESIZE);
    ASSERT(ps > 0, "Failed to get page size");

    return (size_t)ps;
}

typedef struct VirtSpace VirtSpace;

uintptr_t vs_start(VirtSpace*);
size_t vs_reserved(VirtSpace*);
size_t vs_committed(VirtSpace*);
bool vs_exec(VirtSpace*);

static inline
uintptr_t vs_commit_top(VirtSpace* vs) {
    return vs_start(vs) + vs_committed(vs);
}

static inline
uintptr_t vs_end(VirtSpace* vs) {
    return vs_start(vs) + vs_reserved(vs);
}

// If base == 0, then let the system decide the address.
// Otherwise base must be page aligned and the mapping must start exactly there.
// An unavailable range fails without replacing any existing mapping.
// Sizes and accounting are in bytes; reservations are rounded up to whole pages.
VirtSpace* create_virt_space(uintptr_t base, size_t size, bool exec);
void destroy_virt_space(VirtSpace*);

// Returns the committed top before expansion, while 0 if failed.
uintptr_t vs_expand_by(VirtSpace*, size_t, bool touch);
// Returns the committed top after shrinking, while 0 if failed.
uintptr_t vs_shrink_by(VirtSpace*, size_t);

#endif /* CORE_MEMORY_VIRT_SPACE_H_ */
