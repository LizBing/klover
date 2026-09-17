#ifndef GC_GC_H_
#define GC_GC_H_

#include "utils/global_defs.h"

/* Caller serializes initialization. Rejects a live heap or an invalid xmx
 * (not word-aligned, too small for the null sentinel, or over 32 GiB).
 * Failed allocation leaves the heap uninitialized and may be retried. */
bool gc_init(size_t xmx);

#endif /* GC_GC_H_ */
