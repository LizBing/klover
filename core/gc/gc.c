#include "gc/gc.h"
#include "gc/gc_heap.h"

void gc_init(size_t xmx) {
    gcheap_init(xmx);
}
