#include "metaspace/metaspace.h"
#include "memory/comp_space_defs.h"
#include "memory/virt_space.h"
#include "utils/global_defs.h"

#include <stdint.h>
#include <stdlib.h>
#include <pthread.h>

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

/* Normal (small) chunk: 8 KB */
static const size_t SMALL_CHUNK_BYTE_SIZE = 8 * K;

/* -------------------------------------------------------------------------- */
/*  Lock-free stack (Treiber stack)                                           */
/* -------------------------------------------------------------------------- */

typedef struct {
    MSChunk* top;
} LFStack;

static inline void lfstack_push(LFStack* s, MSChunk* chunk) {
    MSChunk* old_top;
    do {
        old_top = __atomic_load_n(&s->top, __ATOMIC_RELAXED);
        chunk->_next = old_top;
    } while (!__atomic_compare_exchange_n(
        &s->top, &old_top, chunk,
        /* weak */ false,
        __ATOMIC_RELEASE,
        __ATOMIC_RELAXED));
}

static inline MSChunk* lfstack_pop(LFStack* s) {
    MSChunk* old_top;
    MSChunk* new_top;
    do {
        old_top = __atomic_load_n(&s->top, __ATOMIC_RELAXED);
        if (old_top == NULL) return NULL;
        new_top = old_top->_next;
    } while (!__atomic_compare_exchange_n(
        &s->top, &old_top, new_top,
        /* weak */ false,
        __ATOMIC_ACQUIRE,
        __ATOMIC_RELAXED));
    return old_top;
}

/* -------------------------------------------------------------------------- */
/*  Metaspace internal state                                                  */
/* -------------------------------------------------------------------------- */

static VirtSpace* _vs        = NULL;
static LFStack    _free_stack;       /* free list for small chunks */
static pthread_mutex_t _mutex = PTHREAD_MUTEX_INITIALIZER;

/* -------------------------------------------------------------------------- */
/*  Round up to the next multiple of 8K                                       */
/* -------------------------------------------------------------------------- */

static inline size_t round_up_8k(size_t byte_size) {
    return (byte_size + SMALL_CHUNK_BYTE_SIZE - 1) & ~(SMALL_CHUNK_BYTE_SIZE - 1);
}

/* -------------------------------------------------------------------------- */
/*  Public API                                                                */
/* -------------------------------------------------------------------------- */

bool ms_init(void) {
    if (_vs != NULL) {
        return false; /* already initialised */
    }

    VirtSpace* vs = create_virt_space(METASPACE_BASE, COMPSPACE_WORD_SIZE, false);
    if (vs == NULL) {
        return false;
    }

    _vs        = vs;
    _free_stack.top = NULL;

    return true;
}

/* -------------------------------------------------------------------------- */

MSChunk* alloc_chunk(size_t byte_size) {
    pthread_mutex_lock(&_mutex);
    
    uintptr_t cstart = (uintptr_t)_vs->commit_top;
    if (!vs_expand(_vs, byte_size / sizeof(HeapWord), false)) {
        return NULL;
    }

    pthread_mutex_unlock(&_mutex);

    MSChunk* c = malloc(sizeof(MSChunk));
    if (!c) return NULL;

    c->_next = NULL;
    c->byte_size = byte_size;
    c->start = cstart;

    return c;
}

MSChunk* ms_alloc_small_chunk(void) {
    /* 1. Try the lock-free free stack first */
    MSChunk* chunk = lfstack_pop(&_free_stack);
    if (chunk != NULL) {
        return chunk;
    }

    return alloc_chunk(SMALL_CHUNK_BYTE_SIZE);
}

/* -------------------------------------------------------------------------- */

MSChunk* ms_alloc_sized_chunk(size_t byte_size) {
    /* Minimum chunk size is sizeof(MSChunk) */
    if (byte_size < sizeof(MSChunk)) {
        byte_size = sizeof(MSChunk);
    }

    /* Round up to the next multiple of 8K */
    size_t aligned = round_up_8k(byte_size);

    return alloc_chunk(aligned);
}

/* -------------------------------------------------------------------------- */

void ms_free_chunk(MSChunk* chunk) {
    if (chunk == NULL) return;  /* matching free(NULL) semantics */

    /* All freed chunks are pushed to the lock-free stack.
     * Small chunks are reused by ms_alloc_small_chunk;
     * large chunks remain on the stack for potential future reuse. */
    lfstack_push(&_free_stack, chunk);
}
