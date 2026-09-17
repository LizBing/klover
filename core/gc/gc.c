#include "gc/gc.h"
#include "gc/gc_heap.h"

bool gc_init(size_t xmx) {
    return gcheap_init(xmx);
}

