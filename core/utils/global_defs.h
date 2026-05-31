#ifndef CORE_UTIlS_GLOBAL_DEFS_H_
#define CORE_UTIlS_GLOBAL_DEFS_H_

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct WordImpl WordImpl;
typedef WordImpl* Word;
typedef Word HeapWord;

static const size_t K = 1024;
static const size_t M = 1024 * K;
static const size_t G = 1024 * M;


#endif /* CORE_UTIlS_GLOBAL_DEFS_H_ */
