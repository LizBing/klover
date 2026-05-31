#ifndef CORE_METASPACE_METASPACE_H_
#define CORE_METASPACE_METASPACE_H_

#include "core/utils/global_defs.h"

typedef struct MSChunk MSChunk;
struct MSChunk {
    MSChunk* _next;
    
    size_t byte_size;
    uintptr_t start;
};

bool ms_init();

MSChunk* ms_alloc_small_chunk();
MSChunk* ms_alloc_sized_chunk(size_t byte_size);
void ms_free_chunk(MSChunk*);

#endif /* CORE_METASPACE_METASPACE_H_ */