#ifndef OOPS_MARKWORD_H_
#define OOPS_MARKWORD_H_

#include "memory/comp_space_defs.h"
#include "utils/global_defs.h"
#include <stdint.h>

static const int LOCKVALUE_NONE = 0x01;
static const int LOCKVALUE_GC = 0x11;
// Light-Weight Lock
// static const int LOCKVALUE_LW = 0x00;
// Heavy-Weight Lock
// static const int LOCKVALUE_HW = 0x00;

static const int LOCKVALUE_BITS = 2;
static const int LOCKVALUE_SHIFT = 0;
static const uint64_t LOCKVALUE_MASK = (1ul << LOCKVALUE_BITS) - 1;
static const uint64_t LOCKVALUE_MASK_IN_PLACE = LOCKVALUE_MASK << LOCKVALUE_SHIFT;

static inline int mw_read_lock_value(uint64_t raw) {
  return (raw & LOCKVALUE_MASK_IN_PLACE) >> LOCKVALUE_SHIFT;
}

static inline uint64_t mw_write_lock_value(uint64_t raw, int value) {
  return (raw & ~LOCKVALUE_MASK_IN_PLACE) | (((uint64_t)value) << LOCKVALUE_SHIFT);
}

static const int KLASS_COMPPTR_BITS = 32;
static const int KLASS_COMPPTR_SHIFT = 31;
static const uint64_t KLASS_COMPPTR_MASK = (1ul << KLASS_COMPPTR_BITS) - 1;
static const uint64_t KLASS_COMPPTR_MASK_IN_PLACE = KLASS_COMPPTR_MASK << KLASS_COMPPTR_SHIFT;

static inline comp_ptr_t mw_read_klass_comp_ptr(uint64_t raw) {
  return (raw & KLASS_COMPPTR_MASK_IN_PLACE) >> KLASS_COMPPTR_SHIFT;
}

static inline uint64_t mw_write_klass_comp_ptr(uint64_t raw, comp_ptr_t value) {
  return (raw & ~KLASS_COMPPTR_MASK_IN_PLACE) | (((uint64_t)value) << KLASS_COMPPTR_SHIFT);
}

static inline uint64_t mw_default(comp_ptr_t kcomp) {
    uint64_t res = mw_write_lock_value(0, LOCKVALUE_NONE);
    res = mw_write_klass_comp_ptr(res, kcomp);

    return res;
}

#endif /* OOPS_MARKWORD_H_ */
