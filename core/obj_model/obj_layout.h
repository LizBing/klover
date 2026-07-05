#ifndef OBJ_MODEL_OBJ_LAYOUT_H_
#define OBJ_MODEL_OBJ_LAYOUT_H_

#include <stddef.h>

/*
 * Mirror of Rust `oops::obj_layout::ObjLayout` (`#[repr(C)]`).
 *
 * Allocated inline inside `NormalKlass` (metaspace).  C-side GC obtains a
 * pointer via `klover_obj_layout_of(klass)`; do NOT cast `Klass*` directly
 * because `Klass` is a Rust enum whose discriminant placement is not stable.
 *
 * Layout semantics (for ordinary objects):
 *
 *   obj start ─────────────────────┐
 *   │  markword (8B)               │  offset 0
 *   ├──────────────────────────────┤
 *   │  super layer                 │  offset 8 (= HEADER_BYTES)
 *   │  ├ oop region (ptr_count×4B) │  ← GC scans here (per layer)
 *   │  ├ padding to 8B if needed   │
 *   │  ├ 8B fields                 │
 *   │  ├ 4B fields                 │
 *   │  ├ 2B fields                 │
 *   │  └ 1B fields                 │
 *   ├──────────────────────────────┤  super.byte_size  ← this layer start
 *   │  this layer                  │
 *   │  ├ oop region (ptr_count×4B) │
 *   │  ├ ...                       │
 *   └──────────────────────────────┘  byte_size (8B-aligned, cumulative)
 *
 * GC iteration (from the most-derived `ObjLayout`, walking up via super_layout):
 *
 *   for (layout = klover_obj_layout_of(klass); layout != NULL;
 *        layout = layout->super_layout) {
 *       size_t layer_start = layout->super_layout
 *                                ? layout->super_layout->byte_size
 *                                : 8;                      // skip markword
 *       for (size_t i = 0; i < layout->ptr_count; i++) {
 *           nobjptr_t* slot = (nobjptr_t*)((char*)obj + layer_start + i * 4);
 *           if (*slot != 0) { ... visit decode(*slot) ... }
 *       }
 *   }
 */
typedef struct ObjLayout ObjLayout;
struct ObjLayout {
    const ObjLayout* super_layout; /* NULL for java.lang.Object */
    size_t byte_size;              /* cumulative: markword + all supers + self */
    size_t ptrs_count;              /* oop count in THIS class's layer only */
};

/* Defined in Rust (src/gc_binding/gc_binding.rs).  Returns NULL for
 * non-Normal klasses (arrays / primitives). */
const ObjLayout* klover_obj_layout_of(const void* klass);

#endif /* OBJ_MODEL_OBJ_LAYOUT_H_ */
