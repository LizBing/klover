#include "gc/gc_heap.h"
#include "memory/comp_space_defs.h"
#include "obj_model/markword.h"

#include "tests/test_harness.h"

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

/* ================================================================== */
/*  Test cases                                                        */
/* ================================================================== */

/* ---- init --------------------------------------------------------- */

TEST(init_basic)
{
    /* Heap was initialized in main(); reaching here means success. */
    ASSERT_TRUE(1, "unreachable");
}

/* ---- single allocation -------------------------------------------- */

TEST(alloc_single)
{
    objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE + 64), 16);
    ASSERT_NOT_NULL(obj, "gcheap_alloc returned NULL");
}

/* ---- markword klass encoding -------------------------------------- */

TEST(alloc_markword_klass_roundtrip)
{
    /*
     * gcheap_alloc encodes klass_ptr via comp_ptr_encode(METASPACE_BASE, ...)
     * and writes the compressed value into the object's markword.
     *
     * Use METASPACE_BASE + 128: encodes to (128 >> 3) = 16, well within
     * the 32-bit compressed pointer range.
     */
    Klass* k = (Klass*)(METASPACE_BASE + 128);
    objptr_t obj = gcheap_alloc(k, 16);
    ASSERT_NOT_NULL(obj, "gcheap_alloc returned NULL");

    comp_ptr_t kcomp = mw_read_klass_comp_ptr(obj->markword);
    ASSERT_TRUE(kcomp != 0, "compressed klass pointer should be non-zero");

    Klass* decoded = (Klass*)comp_ptr_decode(METASPACE_BASE, kcomp);
    ASSERT_EQ(decoded, k, "klass pointer encode/decode roundtrip failed");
}

TEST(alloc_non_null_klass)
{
    /*
     * Even with a low (process-space) pointer value the compressed
     * klass should be non-zero, because klass != NULL means the object
     * has a real type.
     */
    objptr_t obj = gcheap_alloc((Klass*)0x1000, 16);
    ASSERT_NOT_NULL(obj, "gcheap_alloc returned NULL");

    comp_ptr_t kcomp = mw_read_klass_comp_ptr(obj->markword);
    ASSERT_TRUE(kcomp != 0,
                "compressed klass pointer should be non-zero for non-NULL klass");
}

/* ---- sequential allocations (bump-pointer semantics) --------------- */

TEST(alloc_increasing_addresses)
{
    objptr_t a = gcheap_alloc((Klass*)(METASPACE_BASE + 256), 32);
    objptr_t b = gcheap_alloc((Klass*)(METASPACE_BASE + 256), 32);
    ASSERT_NOT_NULL(a, "alloc a returned NULL");
    ASSERT_NOT_NULL(b, "alloc b returned NULL");

    ASSERT_TRUE((HeapWord*)b > (HeapWord*)a,
                "bump allocator should return increasing addresses");
}

TEST(alloc_non_overlapping)
{
    size_t ws = 64;
    objptr_t a = gcheap_alloc((Klass*)(METASPACE_BASE + 256), ws);
    objptr_t b = gcheap_alloc((Klass*)(METASPACE_BASE + 256), ws);
    ASSERT_NOT_NULL(a, "alloc a returned NULL");
    ASSERT_NOT_NULL(b, "alloc b returned NULL");

    ptrdiff_t gap = (HeapWord*)b - (HeapWord*)a;
    ASSERT_TRUE(gap >= (ptrdiff_t)ws,
                "consecutive objects should not overlap");
}

TEST(alloc_many_sequential)
{
    objptr_t objs[100];
    for (int i = 0; i < 100; i++) {
        objs[i] = gcheap_alloc((Klass*)(METASPACE_BASE + 512), 8);
        ASSERT_NOT_NULL(objs[i], "alloc in loop returned NULL");
    }

    for (int i = 1; i < 100; i++) {
        ASSERT_TRUE((HeapWord*)objs[i] > (HeapWord*)objs[i - 1],
                    "sequential allocs should be strictly increasing");
    }
}

/* ---- OOM handling ------------------------------------------------- */

TEST(alloc_oom_huge)
{
    /*
     * Request far more words than the heap committed size can provide.
     * Use 10 Giga-words (80 GB) against a 4 MB heap -> must return NULL.
     */
    objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE), (size_t)10 * G);
    ASSERT_NULL(obj, "huge allocation should return NULL (OOM)");
}

/* ---- memory accessibility ----------------------------------------- */

TEST(alloc_memory_writable)
{
    objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE + 256), 16);
    ASSERT_NOT_NULL(obj, "gcheap_alloc returned NULL");

    /*
     * Write a pattern into the payload area to verify the allocated
     * memory is backed by physical pages and accessible.
     *
     * (The markword sits at offset 0; payload starts right after.)
     */
    for (size_t i = 0; i < 16; i++) {
        obj->payload[i] = (HeapWord)(uintptr_t)(0xCAFE0000 + i);
    }

    /* Read back */
    for (size_t i = 0; i < 16; i++) {
        ASSERT_EQ(obj->payload[i],
                  (HeapWord)(uintptr_t)(0xCAFE0000 + i),
                  "payload readback mismatch");
    }
}

/* ---- sized allocations -------------------------------------------- */

TEST(alloc_varying_sizes)
{
    size_t sizes[] = { 1, 2, 4, 8, 13, 64, 128, 256, 1024 };
    objptr_t prev  = NULL;

    for (int i = 0; i < 9; i++) {
        objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE + 1024), sizes[i]);
        ASSERT_NOT_NULL(obj, "varying-size alloc returned NULL");

        if (prev != NULL) {
            ptrdiff_t gap = (HeapWord*)obj - (HeapWord*)prev;
            ASSERT_TRUE(gap >= (ptrdiff_t)sizes[i - 1],
                        "objects of different sizes should not overlap");
        }
        prev = obj;
    }
}

/* ---- compressed klass edge cases ---------------------------------- */

TEST(alloc_markword_lock_value)
{
    /*
     * A freshly allocated object should have LOCKVALUE_NONE (0x01)
     * encoded in the lock bits of its markword.
     */
    objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE + 64), 16);
    ASSERT_NOT_NULL(obj, "gcheap_alloc returned NULL");

    int lv = mw_read_lock_value(obj->markword);
    ASSERT_EQ(lv, 0x01, "fresh object should have LOCKVALUE_NONE (0x01)");
}

/* ================================================================== */
/*  Runner                                                            */
/* ================================================================== */

int main(void)
{
    printf("\n=== gc_heap tests ===\n\n");

    /*
     * Initialize a small GC heap (4 MB).  The underlying virtual space
     * still reserves the full compressed-pointer addressable range
     * (COMPSPACE_WORD_SIZE), but only 4 MB are committed.
     */
    bool ok = gcheap_init(4 * M);
    if (!ok) {
        printf("  FAILED: gcheap_init() returned false\n");
        return EXIT_FAILURE;
    }

    RUN_TEST(init_basic);
    RUN_TEST(alloc_single);
    RUN_TEST(alloc_markword_klass_roundtrip);
    RUN_TEST(alloc_non_null_klass);
    RUN_TEST(alloc_increasing_addresses);
    RUN_TEST(alloc_non_overlapping);
    RUN_TEST(alloc_many_sequential);
    RUN_TEST(alloc_oom_huge);
    RUN_TEST(alloc_memory_writable);
    RUN_TEST(alloc_varying_sizes);
    RUN_TEST(alloc_markword_lock_value);

    printf("\n---\n");
    printf("  TOTAL:  %d\n", _tests_run);
    printf("  PASSED: %d\n", _tests_pass);
    printf("  FAILED: %d\n\n", _tests_fail);

    return _tests_fail > 0 ? EXIT_FAILURE : EXIT_SUCCESS;
}
