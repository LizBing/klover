#include "gc/access_barrier.h"
#include "memory/comp_space_defs.h"
#include "obj_model/oop_hierarchy.h"
#include <stdatomic.h>
#include <stdint.h>

objptr_t gc_heap_ref_load(nobjptr_t* slot, GCRefAccMode mode) {
    nobjptr_t n = 0;
    switch (mode) {
    case GC_REF_ACC_MODE_PLAIN:
        n = *slot;
        break;

    case GC_REF_ACC_MODE_VOLATILE:
        n = atomic_load_explicit((_Atomic uint32_t*)slot, memory_order_relaxed);
        break;
    }

    return (objptr_t)comp_ptr_decode(GCHEAP_BASE, n);
}

void gc_heap_ref_store(nobjptr_t *slot, objptr_t value, GCRefAccMode mode) {
    nobjptr_t n = comp_ptr_encode(GCHEAP_BASE, value);

    switch (mode) {
    case GC_REF_ACC_MODE_PLAIN:
        *slot = n;
    case GC_REF_ACC_MODE_VOLATILE:
        atomic_store_explicit((_Atomic uint32_t*)slot, n, memory_order_relaxed);
    }
}

objptr_t gc_heap_ref_load_at(objptr_t p, offs_t offs, GCRefAccMode mode) {
    nobjptr_t* slot = (nobjptr_t*)((uintptr_t)p + offs);
    return gc_heap_ref_load(slot, mode);
}

void gc_heap_ref_store_at(objptr_t p, offs_t offs, objptr_t value, GCRefAccMode mode) {
    nobjptr_t* slot = (nobjptr_t*)((uintptr_t)p + offs);
    gc_heap_ref_store(slot, value, mode);
}

#define gc_heap_load_n_at(dst, p, offs, n_type, mode) do {\
    n_type* dst_slot = dst;\
    n_type* src_slot = (n_type*)((uintptr_t)p + offs);\
\
    switch (mode) {\
        case GC_REF_ACC_MODE_PLAIN: {\
            *dst_slot = *src_slot;\
            break;\
        }\
        case GC_REF_ACC_MODE_VOLATILE: {\
            n_type value = atomic_load_explicit((_Atomic n_type*)src_slot, memory_order_relaxed);\
            atomic_store_explicit((_Atomic n_type*)dst_slot, value, memory_order_relaxed);\
        }\
    }\
} while (false)

void gc_heap_load_1_at(void* dst, objptr_t p, offs_t offs, GCRefAccMode mode) {
    gc_heap_load_n_at(dst, p, offs, int8_t, mode);
}

void gc_heap_load_2_at(void* dst, objptr_t p, offs_t offs, GCRefAccMode mode) {
    gc_heap_load_n_at(dst, p, offs, int16_t, mode);
}

void gc_heap_load_4_at(void* dst, objptr_t p, offs_t offs, GCRefAccMode mode) {
    gc_heap_load_n_at(dst, p, offs, int32_t, mode);
}

void gc_heap_load_8_at(void* dst, objptr_t p, offs_t offs, GCRefAccMode mode) {
    gc_heap_load_n_at(dst, p, offs, int64_t, mode);
}

#define gc_heap_store_n_at(p, offs, src, n_type, mode) do {\
    n_type* dst_slot = (n_type*)((uintptr_t)p + offs);\
    n_type* src_slot = (n_type*)src;\
\
    switch (mode) {\
        case GC_REF_ACC_MODE_PLAIN: {\
            *dst_slot = *src_slot;\
            break;\
        }\
        case GC_REF_ACC_MODE_VOLATILE: {\
            n_type value = atomic_load_explicit((_Atomic n_type*)src_slot, memory_order_relaxed);\
            atomic_store_explicit((_Atomic n_type*)dst_slot, value, memory_order_relaxed);\
        }\
    }\
} while (false)

void gc_heap_store_1_at(objptr_t p, offs_t offs, void* src, GCRefAccMode mode) {
    gc_heap_store_n_at(p, offs, src, int8_t, mode);
}

void gc_heap_store_2_at(objptr_t p, offs_t offs, void* src, GCRefAccMode mode) {
    gc_heap_store_n_at(p, offs, src, int16_t, mode);
}

void gc_heap_store_4_at(objptr_t p, offs_t offs, void* src, GCRefAccMode mode) {
    gc_heap_store_n_at(p, offs, src, int32_t, mode);
}

void gc_heap_store_8_at(objptr_t p, offs_t offs, void* src, GCRefAccMode mode) {
    gc_heap_store_n_at(p, offs, src, int64_t, mode);
}
