#ifndef CORE_OOPS_OBJ_DESC_H_
#define CORE_OOPS_OBJ_DESC_H_

#include <stddef.h>
#include <stdatomic.h>

typedef struct ObjDesc ObjDesc;
struct ObjDesc {
  uint64_t markword;
};

#endif /* CORE_OOPS_OBJ_DESC_H_ */

