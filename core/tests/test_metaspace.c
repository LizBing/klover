#include "metaspace/metaspace.h"
#include "memory/virt_space.h"

#include "tests/test_harness.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ================================================================== */
/*  Helper: get usable data pointer from a chunk                       */
/* ================================================================== */

/*
 * In the current metaspace design, the MSChunk struct is separately
 * heap-allocated (via malloc).  The actual chunk data lives in the
 * virtual space at the address stored in c->start.  The usable size
 * is c->byte_size.
 */
static inline void* chunk_data(MSChunk* c) {
    return (void*)c->start;
}

/* ================================================================== */
/*  Test cases                                                        */
/* ================================================================== */

/* ---- init -------------------------------------------------------- */

TEST(init_ok)
{
    /* ms_init() is called once in main(); verify we can proceed */
    ASSERT_TRUE(1, "unreachable");
}

TEST(init_double_fails)
{
    /* Second init must fail (already initialized) */
    bool ok = ms_init();
    ASSERT_TRUE(!ok, "second ms_init should return false");
}

/* ---- small chunk allocation -------------------------------------- */

TEST(alloc_small_chunk_basic)
{
    MSChunk* c = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(c, "ms_alloc_small_chunk returned NULL");
    ASSERT_TRUE(c->byte_size >= 8 * K, "small chunk byte_size too small");
    ASSERT_NOT_NULL((void*)c->start, "chunk start address is NULL");
}

TEST(alloc_small_chunk_multiple)
{
    MSChunk* a = ms_alloc_small_chunk();
    MSChunk* b = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(a, "first alloc returned NULL");
    ASSERT_NOT_NULL(b, "second alloc returned NULL");
    ASSERT_TRUE(a != b, "two small chunks have the same address");
}

TEST(small_chunk_read_write)
{
    MSChunk* c = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(c, "alloc returned NULL");

    void*  data   = chunk_data(c);
    size_t usable = c->byte_size;

    /* Write pattern across the entire usable area */
    memset(data, 0xAB, usable);

    /* Verify first and last bytes */
    ASSERT_EQ(((unsigned char*)data)[0],
              (unsigned char)0xAB, "first byte mismatch");
    ASSERT_EQ(((unsigned char*)data)[usable - 1],
              (unsigned char)0xAB, "last byte mismatch");
}

/* ---- free and reuse (lock-free stack) ---------------------------- */

TEST(free_and_reuse_small_chunk)
{
    MSChunk* a = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(a, "alloc returned NULL");

    /* Free it (ms_free_chunk returns void) */
    ms_free_chunk(a);

    /* Allocate again -- should get the same chunk back from stack */
    MSChunk* b = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(b, "re-alloc returned NULL");
    ASSERT_EQ(a, b, "reused chunk should be the same MSChunk pointer");
}

TEST(free_null_is_safe)
{
    /* ms_free_chunk(NULL) should not crash */
    ms_free_chunk(NULL);
}

/* ---- sized chunk allocation -------------------------------------- */

TEST(alloc_sized_chunk_exact_8k)
{
    MSChunk* c = ms_alloc_sized_chunk(8 * K);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(8K) returned NULL");
    ASSERT_TRUE(c->byte_size >= 8 * K, "byte_size too small");

    /* Write to the usable area */
    void* data = chunk_data(c);
    memset(data, 0xCD, c->byte_size);
}

TEST(alloc_sized_chunk_rounds_up)
{
    /* Request 9K -- should be rounded up to the next 8K boundary (16K) */
    MSChunk* c = ms_alloc_sized_chunk(9 * K);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(9K) returned NULL");
    ASSERT_EQ(c->byte_size, (size_t)(16 * K), "byte_size should be 16K");

    /* Verify the full 16K is usable */
    void* data = chunk_data(c);
    memset(data, 0xEF, c->byte_size);
}

TEST(alloc_sized_chunk_large)
{
    /* Allocate a 2 MB chunk */
    MSChunk* c = ms_alloc_sized_chunk(2 * M);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(2M) returned NULL");
    ASSERT_EQ(c->byte_size, (size_t)(2 * M), "byte_size should be 2M");

    void* data = chunk_data(c);
    memset(data, 0x42, c->byte_size);
}

TEST(alloc_sized_chunk_minimum)
{
    /* Very small request -- should be clamped to sizeof(MSChunk)
     * and then rounded up to 8K. */
    MSChunk* c = ms_alloc_sized_chunk(1);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(1) returned NULL");
    ASSERT_TRUE(c->byte_size >= 8 * K, "minimum chunk should be at least 8K");
}

/* ---- free any chunk ---------------------------------------------- */

TEST(free_large_chunk_no_crash)
{
    MSChunk* c = ms_alloc_sized_chunk(16 * K);
    ASSERT_NOT_NULL(c, "alloc returned NULL");

    /* Freeing a large chunk should not crash; ms_free_chunk returns void */
    ms_free_chunk(c);
}

/* ---- unified free stack behaviour --------------------------------- */

/*
 * In the current design, ALL freed chunks (small and large) go to a
 * single lock-free LIFO stack.  Therefore a freed large chunk CAN be
 * returned by ms_alloc_small_chunk.
 */
TEST(freed_large_reused_by_small_alloc)
{
    /* Allocate and free a large chunk */
    MSChunk* large = ms_alloc_sized_chunk(16 * K);
    ASSERT_NOT_NULL(large, "alloc large returned NULL");
    ms_free_chunk(large);

    /* Small alloc pops from the stack -- should get the large chunk */
    MSChunk* small = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(small, "alloc small returned NULL");
    ASSERT_EQ(large, small,
              "freed large chunk should be reused by small alloc");
}

TEST(stack_lifo_order)
{
    /* Allocate two distinct small chunks */
    MSChunk* a = ms_alloc_small_chunk();
    MSChunk* b = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(a, "alloc a");
    ASSERT_NOT_NULL(b, "alloc b");
    ASSERT_TRUE(a != b, "chunks should be distinct");

    /* Free both: b last, so b is on top of stack */
    ms_free_chunk(a);
    ms_free_chunk(b);

    /* First alloc should pop b (LIFO) */
    MSChunk* c = ms_alloc_small_chunk();
    ASSERT_EQ(b, c, "should pop b first (LIFO order)");

    /* Second alloc should pop a */
    MSChunk* d = ms_alloc_small_chunk();
    ASSERT_EQ(a, d, "should pop a second");
}

/* ---- sizing round-up edge cases ---------------------------------- */

TEST(alloc_sized_chunk_already_aligned)
{
    /* 32K is already a multiple of 8K -- no round-up needed */
    MSChunk* c = ms_alloc_sized_chunk(32 * K);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(32K) returned NULL");
    ASSERT_EQ(c->byte_size, (size_t)(32 * K), "byte_size should be 32K");

    void* data = chunk_data(c);
    memset(data, 0x55, c->byte_size);
}

TEST(alloc_sized_chunk_one_byte_over_8k)
{
    /* 8K + 1 -> rounds up to 16K */
    MSChunk* c = ms_alloc_sized_chunk((8 * K) + 1);
    ASSERT_NOT_NULL(c, "alloc_sized_chunk(8K+1) returned NULL");
    ASSERT_EQ(c->byte_size, (size_t)(16 * K), "byte_size should be 16K");
}

/* ---- chunk independence ------------------------------------------ */

TEST(chunks_have_distinct_data_regions)
{
    MSChunk* a = ms_alloc_small_chunk();
    MSChunk* b = ms_alloc_small_chunk();
    ASSERT_NOT_NULL(a, "alloc a");
    ASSERT_NOT_NULL(b, "alloc b");

    void* data_a = chunk_data(a);
    void* data_b = chunk_data(b);

    /* Write distinct patterns */
    memset(data_a, 0x11, a->byte_size);
    memset(data_b, 0x22, b->byte_size);

    /* Verify no overlap: first byte of each should retain its pattern */
    ASSERT_EQ(((unsigned char*)data_a)[0], (unsigned char)0x11,
              "chunk a pattern corrupted");
    ASSERT_EQ(((unsigned char*)data_b)[0], (unsigned char)0x22,
              "chunk b pattern corrupted");

    /* The data regions should not overlap */
    ASSERT_TRUE(
        (uintptr_t)data_b >= (uintptr_t)data_a + a->byte_size ||
        (uintptr_t)data_a >= (uintptr_t)data_b + b->byte_size,
        "chunk data regions overlap");
}

/* ================================================================== */
/*  Runner                                                            */
/* ================================================================== */

int main(void)
{
    printf("\n=== metaspace tests ===\n\n");

    /* Initialise metaspace once for all tests */
    bool ok = ms_init();
    if (!ok) {
        printf("  FAILED: ms_init() returned false\n");
        return EXIT_FAILURE;
    }

    RUN_TEST(init_ok);
    RUN_TEST(init_double_fails);
    RUN_TEST(alloc_small_chunk_basic);
    RUN_TEST(alloc_small_chunk_multiple);
    RUN_TEST(small_chunk_read_write);
    RUN_TEST(free_and_reuse_small_chunk);
    RUN_TEST(free_null_is_safe);
    RUN_TEST(alloc_sized_chunk_exact_8k);
    RUN_TEST(alloc_sized_chunk_rounds_up);
    RUN_TEST(alloc_sized_chunk_large);
    RUN_TEST(alloc_sized_chunk_minimum);
    RUN_TEST(free_large_chunk_no_crash);
    RUN_TEST(freed_large_reused_by_small_alloc);
    RUN_TEST(stack_lifo_order);
    RUN_TEST(alloc_sized_chunk_already_aligned);
    RUN_TEST(alloc_sized_chunk_one_byte_over_8k);
    RUN_TEST(chunks_have_distinct_data_regions);

    printf("\n---\n");
    printf("  TOTAL:  %d\n", _tests_run);
    printf("  PASSED: %d\n", _tests_pass);
    printf("  FAILED: %d\n\n", _tests_fail);

    return _tests_fail > 0 ? EXIT_FAILURE : EXIT_SUCCESS;
}
