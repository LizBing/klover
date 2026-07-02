#ifndef GC_GC_HEAP_H_
#define GC_GC_HEAP_H_

#include "obj_model/oop_hierarchy.h"
#include "utils/global_defs.h"

// xmx: byte size
bool gcheap_init(size_t xmx);

// Returning NULL means OOM.
oop_t gcheap_alloc(Klass*, size_t word_size);

#endif /* GC_GC_HEAP_H_ */
