#ifndef OBJ_MODEL_OBJ_ACCESS_H_
#define OBJ_MODEL_OBJ_ACCESS_H_

#include "memory/comp_space_defs.h"
#include "obj_model/markword.h"
#include "obj_model/oop_hierarchy.h"

#include <string.h>

/* -------------------------------------------------------------------------- */
/*  Object layout constants                                                   */
/*                                                                           */
/*  Regular object:                                                           */
/*    ┌──────────────┬─────────────────────────────────────────┐              */
/*    │  markword    │  fields ...                              │              */
/*    │  8 bytes     │                                          │              */
/*    └──────────────┴─────────────────────────────────────────┘              */
/*    offset 0        offset 8                                               */
/*                                                                           */
/*  Array object:                                                            */
/*    ┌──────────────┬────────────┬──────────┬──────────┬─────┐              */
/*    │  markword    │  length    │  elem[0] │  elem[1] │ ... │              */
/*    │  8 bytes     │  4 bytes   │          │          │     │              */
/*    └──────────────┴────────────┴──────────┴──────────┴─────┘              */
/*    offset 0        offset 8     offset 16                                 */
/*                                                                           */
/*  Memory model:                                                             */
/*    - Inside objects, oop fields are stored as 4-byte noop_t.              */
/*    - Outside objects (locals, OopStorage slots, etc.), oops are oop_t.     */
/*    - The public API encodes/decodes transparently.                         */
/*                                                                           */
/*  All offsets are byte offsets from the base of the ObjDesc (oop_t).       */
/* -------------------------------------------------------------------------- */

static const size_t OBJ_MARKWORD_OFFSET   = 0;
static const size_t OBJ_PAYLOAD_OFFSET    = 8;   /* first field              */
static const size_t ARRAY_LENGTH_OFFSET   = 8;   /* int32_t length           */
static const size_t ARRAY_DATA_OFFSET     = 16;  /* first element            */

/* ================================================================== */
/*  Markword access                                                    */
/* ================================================================== */

static inline uint64_t obj_markword(oop_t obj) {
    return obj->markword;
}

static inline void obj_set_markword(oop_t obj, uint64_t val) {
    obj->markword = val;
}

/* ================================================================== */
/*  Klass access  (narrow oop in markword)                             */
/* ================================================================== */

static inline noop_t obj_klass(oop_t obj) {
    return mw_read_comp_ptr(obj->markword);
}

static inline void obj_set_klass(oop_t obj, noop_t klass) {
    obj->markword = mw_write_comp_ptr(obj->markword, klass);
}

/* ================================================================== */
/*  Lock state access                                                  */
/* ================================================================== */

static inline int obj_lock_state(oop_t obj) {
    return mw_read_lock_value(obj->markword);
}

static inline void obj_set_lock_state(oop_t obj, int state) {
    obj->markword = mw_write_lock_value(obj->markword, state);
}

/* ================================================================== */
/*  Array length                                                       */
/* ================================================================== */

static inline int32_t obj_array_length(oop_t obj) {
    int32_t len;
    memcpy(&len, (char*)obj + ARRAY_LENGTH_OFFSET, sizeof(len));
    return len;
}

static inline void obj_set_array_length(oop_t obj, int32_t len) {
    memcpy((char*)obj + ARRAY_LENGTH_OFFSET, &len, sizeof(len));
}

/* ================================================================== */
/*  Primitive field access  (offset = byte offset from obj base)      */
/* ================================================================== */

#define DEF_FIELD_ACCESS(name_suffix, c_type)                                  \
    static inline c_type obj_field_##name_suffix(oop_t obj, size_t off) {      \
        c_type v;                                                              \
        memcpy(&v, (char*)obj + off, sizeof(v));                               \
        return v;                                                              \
    }                                                                          \
    static inline void obj_field_put_##name_suffix(oop_t obj, size_t off,      \
                                                   c_type val) {               \
        memcpy((char*)obj + off, &val, sizeof(val));                           \
    }

DEF_FIELD_ACCESS(byte,   int8_t)
DEF_FIELD_ACCESS(short_, int16_t)
DEF_FIELD_ACCESS(int,    int32_t)
DEF_FIELD_ACCESS(long_,  int64_t)
DEF_FIELD_ACCESS(float_, float)
DEF_FIELD_ACCESS(double_,double)

#undef DEF_FIELD_ACCESS

/* Oop field — stored as 4-byte noop_t inside the object,
 * exposed as full oop_t with transparent encode/decode. */
static inline oop_t obj_field_oop(oop_t obj, size_t off) {
    noop_t narrow;
    memcpy(&narrow, (char*)obj + off, sizeof(narrow));
    return comp_ptr_decode(GCHEAP_BASE, narrow);
}

static inline void obj_field_put_oop(oop_t obj, size_t off, oop_t val) {
    noop_t narrow = comp_ptr_encode(GCHEAP_BASE, val);
    memcpy((char*)obj + off, &narrow, sizeof(narrow));
}

/* Convenience aliases for JVM type names */
#define obj_field_char(obj, off)          obj_field_short_(obj, off)
#define obj_field_put_char(obj, off, v)   obj_field_put_short_(obj, off, (int16_t)(v))
#define obj_field_bool(obj, off)          obj_field_byte(obj, off)
#define obj_field_put_bool(obj, off, v)   obj_field_put_byte(obj, off, (int8_t)(v))

/* ================================================================== */
/*  Array element access  (index = element index, 0-based)            */
/* ================================================================== */

#define DEF_ARRAY_ACCESS(name_suffix, c_type)                                  \
    static inline c_type obj_array_##name_suffix(oop_t arr, size_t idx) {      \
        c_type v;                                                              \
        size_t off = ARRAY_DATA_OFFSET + idx * sizeof(c_type);                 \
        memcpy(&v, (char*)arr + off, sizeof(v));                               \
        return v;                                                              \
    }                                                                          \
    static inline void obj_array_put_##name_suffix(oop_t arr, size_t idx,      \
                                                   c_type val) {               \
        size_t off = ARRAY_DATA_OFFSET + idx * sizeof(c_type);                 \
        memcpy((char*)arr + off, &val, sizeof(val));                           \
    }

DEF_ARRAY_ACCESS(byte,   int8_t)
DEF_ARRAY_ACCESS(short_, int16_t)
DEF_ARRAY_ACCESS(int,    int32_t)
DEF_ARRAY_ACCESS(long_,  int64_t)
DEF_ARRAY_ACCESS(float_, float)
DEF_ARRAY_ACCESS(double_,double)

#undef DEF_ARRAY_ACCESS

/* Oop array element — stored as 4-byte noop_t, exposed as oop_t. */
static inline oop_t obj_array_oop(oop_t arr, size_t idx) {
    noop_t narrow;
    size_t off = ARRAY_DATA_OFFSET + idx * sizeof(noop_t);
    memcpy(&narrow, (char*)arr + off, sizeof(narrow));
    return comp_ptr_decode(GCHEAP_BASE, narrow);
}

static inline void obj_array_put_oop(oop_t arr, size_t idx, oop_t val) {
    noop_t narrow = comp_ptr_encode(GCHEAP_BASE, val);
    size_t off = ARRAY_DATA_OFFSET + idx * sizeof(noop_t);
    memcpy((char*)arr + off, &narrow, sizeof(narrow));
}

/* Convenience aliases */
#define obj_array_char(arr, idx)         obj_array_short_(arr, idx)
#define obj_array_put_char(arr, idx, v)  obj_array_put_short_(arr, idx, (int16_t)(v))
#define obj_array_bool(arr, idx)         obj_array_byte(arr, idx)
#define obj_array_put_bool(arr, idx, v)  obj_array_put_byte(arr, idx, (int8_t)(v))

/* ================================================================== */
/*  Raw payload pointer  (for bulk operations / debugging)            */
/* ================================================================== */

/** Return a pointer to the first payload byte (skipping the markword). */
static inline void* obj_payload(oop_t obj) {
    return (char*)obj + OBJ_PAYLOAD_OFFSET;
}

/** Return a pointer to the first array element. */
static inline void* obj_array_data(oop_t obj) {
    return (char*)obj + ARRAY_DATA_OFFSET;
}

#endif /* OBJ_MODEL_OBJ_ACCESS_H_ */
