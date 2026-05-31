#include "core/gc/oop_storage.h"

#include <pthread.h>
#include <stdlib.h>
#include <string.h>

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

#define SLOTS_PER_BLOCK    64u
#define FREE_LIST_END      UINT32_MAX

/* -------------------------------------------------------------------------- */
/*  OopBlock                                                                  */
/*                                                                           */
/*  Allocated as a single contiguous malloc block:                            */
/*                                                                           */
/*    ┌─── OopBlock header ───┬── slot array (_slots[]) ──────────────────┐  */
/*    │ _next, _slot_count,   │ slot[0] │ slot[1] │ ... │ slot[N-1]      │  */
/*    │ _allocated_count,     │ (oop_t) │ (oop_t) │     │ (oop_t)         │  */
/*    │ _free_head,           │ 8 bytes │ 8 bytes │     │ 8 bytes         │  */
/*    │ _next_unused          │         │         │     │                 │  */
/*    └───────────────────────┴─────────┴─────────┴─────┴─────────────────┘  */
/*                                                                           */
/*  Slot lifecycle within a block:                                            */
/*    [0 .. _next_unused)   -- ever-allocated slots (some freed)               */
/*    [_next_unused .. N)   -- never-allocated (virgin)                       */
/*                                                                           */
/*  Free list:  intrusive LIFO using freed slot memory.                       */
/*    Freed slot i stores (uint32_t) next_free_index in its first 4 bytes.    */
/* -------------------------------------------------------------------------- */

typedef struct OopBlock OopBlock;
struct OopBlock {
    OopBlock* _next;
    uint32_t  _slot_count;
    uint32_t  _allocated_count; /* currently live (allocated minus released)   */
    uint32_t  _free_head;       /* FREE_LIST_END if empty                      */
    uint32_t  _next_unused;     /* bump-index for never-allocated slots        */
    oop_t     _slots[];         /* flexible array member                       */
};

/* -------------------------------------------------------------------------- */
/*  OOPStorage                                                                */
/* -------------------------------------------------------------------------- */

typedef struct OOPStorage OOPStorage;
struct OOPStorage {
    OopBlock*       _active;
    OopBlock*       _blocks;
    pthread_mutex_t _mutex;
    bool            _initialized;
    size_t          _total_blocks;
};

/* -------------------------------------------------------------------------- */
/*  Global storages                                                           */
/* -------------------------------------------------------------------------- */

static OOPStorage storages[ALL_STORAGE_COUNT];

/* -------------------------------------------------------------------------- */
/*  Internal helpers                                                          */
/* -------------------------------------------------------------------------- */

static inline uint32_t read_link(oop_t* slot) {
    uint32_t v;
    memcpy(&v, slot, sizeof(v));
    return v;
}

static inline void write_link(oop_t* slot, uint32_t next) {
    memcpy(slot, &next, sizeof(next));
}

/* -------------------------------------------------------------------------- */
/*  Block management                                                          */
/* -------------------------------------------------------------------------- */

static OopBlock* block_create(uint32_t slot_count) {
    size_t sz = sizeof(OopBlock) + (size_t)slot_count * sizeof(oop_t);
    OopBlock* blk = (OopBlock*)malloc(sz);
    if (blk == NULL) return NULL;

    blk->_next            = NULL;
    blk->_slot_count      = slot_count;
    blk->_allocated_count = 0;
    blk->_free_head       = FREE_LIST_END;
    blk->_next_unused     = 0;
    memset(blk->_slots, 0, (size_t)slot_count * sizeof(oop_t));
    return blk;
}

static void block_destroy(OopBlock* blk) { free(blk); }

static OopBlock* block_of_slot(const OOPStorage* s, oop_t* slot) {
    for (OopBlock* b = s->_blocks; b; b = b->_next) {
        if (slot >= b->_slots && slot < b->_slots + b->_slot_count) return b;
    }
    return NULL;
}

/* -------------------------------------------------------------------------- */
/*  Slot allocation helpers                                                   */
/* -------------------------------------------------------------------------- */

/** Pop the head of a block's free list.  Returns NULL if empty. */
static oop_t* block_alloc_free(OopBlock* blk) {
    if (blk->_free_head == FREE_LIST_END) return NULL;

    uint32_t idx       = blk->_free_head;
    oop_t*   raw_slot  = (oop_t*)&blk->_slots[idx];
    blk->_free_head    = read_link(raw_slot);
    blk->_allocated_count++;
    *raw_slot = NULL;  /* clear the slot for the caller */
    return &blk->_slots[idx];
}

/** Take the next never-allocated slot.  Returns NULL if full. */
static oop_t* block_alloc_unused(OopBlock* blk) {
    if (blk->_next_unused >= blk->_slot_count) return NULL;

    uint32_t idx = blk->_next_unused++;
    blk->_allocated_count++;
    blk->_slots[idx] = NULL;
    return &blk->_slots[idx];
}

/* -------------------------------------------------------------------------- */
/*  Public API                                                                */
/* -------------------------------------------------------------------------- */

void init_oop_storages(void) {
    for (int i = 0; i < ALL_STORAGE_COUNT; i++) {
        OOPStorage* s = &storages[i];
        if (s->_initialized) continue;

        s->_active       = NULL;
        s->_blocks       = NULL;
        s->_total_blocks = 0;
        s->_initialized  = true;
        pthread_mutex_init(&s->_mutex, NULL);
    }
}

/* -------------------------------------------------------------------------- */

oop_t* alloc_oop_slot(int storage_id) {
    if (storage_id < 0 || storage_id >= ALL_STORAGE_COUNT) return NULL;

    OOPStorage* s = &storages[storage_id];
    if (!s->_initialized) return NULL;

    pthread_mutex_lock(&s->_mutex);

    oop_t* slot = NULL;

    /* 1. Free list — active block */
    if (s->_active) {
        slot = block_alloc_free(s->_active);
        if (slot) goto done;
    }

    /* 2. Free list — other blocks */
    for (OopBlock* b = s->_blocks; b; b = b->_next) {
        if (b == s->_active) continue;
        slot = block_alloc_free(b);
        if (slot) goto done;
    }

    /* 3. Never-allocated slot from active block */
    if (s->_active) {
        slot = block_alloc_unused(s->_active);
        if (slot) goto done;
    }

    /* 4. New block */
    {
        OopBlock* blk = block_create(SLOTS_PER_BLOCK);
        if (blk == NULL) goto done;

        blk->_next   = s->_blocks;
        s->_blocks   = blk;
        s->_active   = blk;
        s->_total_blocks++;

        slot = block_alloc_unused(blk);  /* takes slot 0 */
    }

done:
    pthread_mutex_unlock(&s->_mutex);
    return slot;
}

/* -------------------------------------------------------------------------- */

void free_oop_slot(int storage_id, oop_t* slot) {
    if (storage_id < 0 || storage_id >= ALL_STORAGE_COUNT) return;
    if (slot == NULL) return;

    OOPStorage* s = &storages[storage_id];
    if (!s->_initialized) return;

    pthread_mutex_lock(&s->_mutex);

    OopBlock* blk = block_of_slot(s, slot);
    if (blk == NULL) { pthread_mutex_unlock(&s->_mutex); return; }

    uint32_t idx = (uint32_t)(slot - blk->_slots);

    /* Push onto free list.
     * Clear the oop first, then write the link so it survives. */
    *slot = NULL;
    oop_t* raw_slot = (oop_t*)&blk->_slots[idx];
    write_link(raw_slot, blk->_free_head);
    blk->_free_head = idx;

    if (blk->_allocated_count > 0) blk->_allocated_count--;

    pthread_mutex_unlock(&s->_mutex);
}

/* -------------------------------------------------------------------------- */
/*  Iteration                                                                 */
/* -------------------------------------------------------------------------- */

static bool storage_iterate(OOPStorage* s, OOPClosure* closure) {
    if (s == NULL || !s->_initialized) return true;
    if (closure == NULL || closure->func == NULL) return true;

    pthread_mutex_lock(&s->_mutex);

    for (OopBlock* blk = s->_blocks; blk; blk = blk->_next) {
        if (blk->_allocated_count == 0) continue;

        /* Collect free-list indices into a local array */
        uint32_t cap = 64, cnt = 0;
        uint32_t* free_idxs = (uint32_t*)malloc(cap * sizeof(uint32_t));
        if (free_idxs == NULL) continue;

        uint32_t fi = blk->_free_head;
        while (fi != FREE_LIST_END) {
            if (cnt >= cap) {
                cap *= 2;
                uint32_t* tmp = (uint32_t*)realloc(free_idxs, cap * sizeof(uint32_t));
                if (tmp == NULL) break;
                free_idxs = tmp;
            }
            free_idxs[cnt++] = fi;
            oop_t* raw = (oop_t*)&blk->_slots[fi];
            fi = read_link(raw);
        }

        /* Walk [0, _next_unused), skipping freed indices */
        for (uint32_t i = 0; i < blk->_next_unused; i++) {
            bool is_free = false;
            for (uint32_t j = 0; j < cnt; j++) {
                if (free_idxs[j] == i) { is_free = true; break; }
            }
            if (is_free) continue;

            if (!closure->func(&blk->_slots[i], closure->ctx)) {
                free(free_idxs);
                pthread_mutex_unlock(&s->_mutex);
                return false;
            }
        }

        free(free_idxs);
    }

    pthread_mutex_unlock(&s->_mutex);
    return true;
}

bool strong_native_oops_iterate(OOPClosure* closure) {
    for (int i = 0; i < STRONG_STORAGE_COUNT; i++) {
        if (!storage_iterate(&storages[STRONG_STORAGE_START + i], closure))
            return false;
    }
    return true;
}

bool weak_native_oops_iterate(OOPClosure* closure) {
    for (int i = 0; i < WEAK_STORAGE_COUNT; i++) {
        if (!storage_iterate(&storages[WEAK_STORAGE_START + i], closure))
            return false;
    }
    return true;
}

bool all_native_oops_iterate(OOPClosure* closure) {
    for (int i = 0; i < ALL_STORAGE_COUNT; i++) {
        if (!storage_iterate(&storages[i], closure))
            return false;
    }
    return true;
}
