#include "gc/gc_heap.h"
#include "memory/comp_space_defs.h"
#include "memory/virt_space.h"
#include "obj_model/markword.h"
#include "obj_model/oop_hierarchy.h"
#include "utils/global_defs.h"
#include <stdatomic.h>

static VirtSpace* VS = NULL;
static HeapWord* _Atomic BUMPING_TOP = NULL;

bool gcheap_init(size_t xmx) {
    VS = create_virt_space(GCHEAP_BASE, COMPSPACE_WORD_SIZE, false);
    if (VS == NULL) {
        return false;
    }
    vs_expand(VS, xmx / sizeof(HeapWord), false);
    
    BUMPING_TOP = VS->start;

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
    
    return obj;
}
