#include "gc/gc_heap.h"
#include "memory/comp_space_defs.h"
#include "memory/virt_space.h"
#include "obj_model/markword.h"
#include "obj_model/oop_hierarchy.h"
#include "utils/global_defs.h"
#include <stdatomic.h>
#include <string.h>

static VirtSpace* VS = NULL;
static HeapWord* _Atomic BUMPING_TOP = NULL;

bool gcheap_init(size_t xmx) {
    VS = create_virt_space(GCHEAP_BASE, COMPSPACE_WORD_SIZE, false);
    if (VS == NULL) {
        return false;
    }
    vs_expand(VS, xmx / sizeof(HeapWord), false);

    /* Skip the first word so that the first allocation never lands at offset 0
     * – offset 0 is the NULL sentinel for compressed pointers.
     * See comp_space_defs.h: "Zero-sentinel null preservation". */
    BUMPING_TOP = VS->start + 1;

    return true;
}

objptr_t gcheap_alloc(Klass* klass_ptr, size_t word_size) {
    HeapWord* cur_top = atomic_load_explicit(&BUMPING_TOP, memory_order_acquire);
    HeapWord* new_top = NULL;

    for (;;) {
        new_top = cur_top + word_size;
        if (new_top > VS->commit_top) { return NULL; }

        if (atomic_compare_exchange_weak_explicit(&BUMPING_TOP, &cur_top, new_top, memory_order_release, memory_order_acquire)) {
            break;
        }
    }

    objptr_t obj = (objptr_t)cur_top;

    obj->markword = mw_default(comp_ptr_encode(METASPACE_BASE, klass_ptr));

    /* Zero the payload (everything after the markword) so all fields start
     * at their default values (0 / null).  Required by the JVM spec and
     * keeps GC from scanning stale data. */
    size_t payload_bytes = (word_size * sizeof(HeapWord)) - sizeof(ObjDesc);
    memset(obj->payload, 0, payload_bytes);

    return obj;
}
