#include "gc/gc.h"
#include "gc/gc_heap.h"
#include "gc/oop_storage.h"

void gc_init(size_t xmx) {
    gcheap_init(xmx);
    init_oop_storages();
}

