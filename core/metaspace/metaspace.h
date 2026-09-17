#ifndef METASPACE_METASPACE_H_
#define METASPACE_METASPACE_H_

#include "utils/global_defs.h"

typedef struct MSChunk MSChunk;
struct MSChunk {
    MSChunk* _next;
    
    size_t byte_size;
    uintptr_t start;
};

typedef enum {
    MS_INIT_OK                  = 0,
    MS_INIT_ALREADY_INITIALIZED = 1,
    MS_INIT_VSPACE_FAILED       = 2,
} MSInitStatus;

/* Returns an MSInitStatus code with a fixed-width FFI representation.
 * Initialization must be serialized by the caller. */
int32_t c_ms_try_init(void);

MSChunk* ms_alloc_small_chunk();
MSChunk* ms_alloc_sized_chunk(size_t byte_size);
void ms_free_chunk(MSChunk*);

#endif /* METASPACE_METASPACE_H_ */