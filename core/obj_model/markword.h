#ifndef CORE_OOPS_MARKWORD_H_
#define CORE_OOPS_MARKWORD_H_

#include "core/memory/comp_space_defs.h"
#include "core/utils/global_defs.h"

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

static const int COMPPTR_BITS = 32;
static const int COMPPTR_SHIFT = 31;
static const uint64_t COMPPTR_MASK = (1ul << COMPPTR_BITS) - 1;
static const uint64_t COMPPTR_MASK_IN_PLACE = COMPPTR_MASK << COMPPTR_SHIFT;

static inline comp_ptr_t mw_read_comp_ptr(uint64_t raw) {
  return (raw & COMPPTR_MASK_IN_PLACE) >> COMPPTR_SHIFT;
}

static inline uint64_t mw_write_comp_ptr(uint64_t raw, comp_ptr_t value) {
  return (raw & ~COMPPTR_MASK_IN_PLACE) | (((uint64_t)value) << COMPPTR_SHIFT);
}

#endif /* CORE_OOPS_MARKWORD_H_ */
