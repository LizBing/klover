#ifndef CORE_GC_OOP_STORAGE_H_
#define CORE_GC_OOP_STORAGE_H_

#include "core/gc/oop_closure.h"
#include "core/obj_model/oop_hierarchy.h"

static const int STRONG_STORAGE_COUNT = 0;
static const int WEAK_STORAGE_COUNT = 1;
static const int ALL_STORAGE_COUNT = STRONG_STORAGE_COUNT + WEAK_STORAGE_COUNT;
static const int STRONG_STORAGE_START = 0;
static const int WEAK_STORAGE_START = STRONG_STORAGE_START + STRONG_STORAGE_COUNT;

void init_oop_storages();

oop_t* alloc_oop_slot(int storage_id);
void free_oop_slot(int storage_id, oop_t*);

bool strong_native_oops_iterate(OOPClosure*);
bool weak_native_oops_iterate(OOPClosure*);
bool all_native_oops_iterate(OOPClosure*);

#endif /* CORE_GC_OOP_STORAGE_H_ */
