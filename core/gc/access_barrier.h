#ifndef GC_ACCESS_BARRIER_H_
#define GC_ACCESS_BARRIER_H_

#include "obj_model/oop_hierarchy.h"

typedef enum GCRefAccMode GCRefAccMode;
enum GCRefAccMode {
    GC_REF_ACC_MODE_PLAIN,
    GC_REF_ACC_MODE_VOLATILE,
};

objptr_t gc_heap_ref_load(nobjptr_t* slot, GCRefAccMode);
void gc_heap_ref_store(nobjptr_t* slot, objptr_t, GCRefAccMode);

objptr_t gc_heap_ref_load_at(objptr_t, offs_t, GCRefAccMode);
void gc_heap_ref_store_at(objptr_t, offs_t, objptr_t value, GCRefAccMode);

void gc_heap_load_1_at(void* dst, objptr_t, offs_t, GCRefAccMode);
void gc_heap_load_2_at(void* dst, objptr_t, offs_t, GCRefAccMode);
void gc_heap_load_4_at(void* dst, objptr_t, offs_t, GCRefAccMode);
void gc_heap_load_8_at(void* dst, objptr_t, offs_t, GCRefAccMode);

void gc_heap_store_1_at(objptr_t, offs_t, void* src, GCRefAccMode);
void gc_heap_store_2_at(objptr_t, offs_t, void* src, GCRefAccMode);
void gc_heap_store_4_at(objptr_t, offs_t, void* src, GCRefAccMode);
void gc_heap_store_8_at(objptr_t, offs_t, void* src, GCRefAccMode);

#endif /* GC_ACCESS_BARRIER_H_ */
