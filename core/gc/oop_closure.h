#ifndef CORE_GC_OOP_CLOSURE_H_
#define CORE_GC_OOP_CLOSURE_H_

#include "core/obj_model/oop_hierarchy.h"

typedef struct OOPClosure OOPClosure;
struct OOPClosure {
    bool(*func)(oop_t*, void* ctx);
    void* ctx;
};

#endif /* CORE_GC_OOP_CLOSURE_H_ */
