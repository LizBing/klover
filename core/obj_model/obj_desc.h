#ifndef CORE_OOPS_OBJ_DESC_H_
#define CORE_OOPS_OBJ_DESC_H_

#include "utils/global_defs.h"

typedef struct Klass Klass;

typedef struct ObjDesc ObjDesc;
struct ObjDesc {
  uint64_t _Atomic markword;
  HeapWord payload[0];
};

#endif /* CORE_OOPS_OBJ_DESC_H_ */

