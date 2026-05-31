#include "core/memory/virt_space.h"

#include <assert.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ================================================================== */
/*  Minimal test harness                                               */
/* ================================================================== */

static int _tests_run   = 0;
static int _tests_pass  = 0;
static int _tests_fail  = 0;

#define TEST(name)                              \
    static void test_##name(void);              \
    static void test_##name(void)

#define RUN_TEST(name)  do {                                \
        _tests_run++;                                       \
        printf("  %-50s", #name);                           \
        fflush(stdout);                                     \
        test_##name();                                      \
        _tests_pass++;                                      \
        printf("  PASSED\n");                               \
    } while (0)

#define FAIL(msg)  do {                                     \
        printf("  FAILED\n        %s\n", (msg));            \
        _tests_fail++;                                      \
        _tests_pass--;                                      \
        return;                                             \
    } while (0)

#define ASSERT_TRUE(cond, msg)  do { if (!(cond)) FAIL(msg); } while (0)
#define ASSERT_EQ(a, b, msg)    do { if ((a) != (b)) FAIL(msg); } while (0)
#define ASSERT_NOT_NULL(p, msg) do { if ((p) == NULL) FAIL(msg); } while (0)

/* ================================================================== */
/*  Test cases                                                        */
/* ================================================================== */

/* ---- basic lifecycle --------------------------------------------- */

TEST(create_destroy_basic)
{
    VirtSpace* vs = create_virt_space(0, 1024, false);
    ASSERT_NOT_NULL(vs, "create_virt_space returned NULL");
    destroy_virt_space(vs);
}

/* ---- exec flag --------------------------------------------------- */

/*
 * On macOS (Apple Silicon) the hardened runtime enforces W^X:
 * PROT_WRITE | PROT_EXEC is rejected by mmap.  Our implementation
 * always maps READ|WRITE and stores the 'exec' flag separately.
 * Callers that need execute permission must later toggle protection.
 */
TEST(exec_flag_stored)
{
    VirtSpace* vs = create_virt_space(0, 512, true);
    ASSERT_NOT_NULL(vs, "create_virt_space(exec=true) returned NULL");
    ASSERT_TRUE(vs->exec, "expected vs->exec == true");
    destroy_virt_space(vs);
}

TEST(noexec_flag_stored)
{
    VirtSpace* vs = create_virt_space(0, 256, false);
    ASSERT_NOT_NULL(vs, "create_virt_space(exec=false) returned NULL");
    ASSERT_TRUE(!vs->exec, "expected vs->exec == false");
    destroy_virt_space(vs);
}

/* ---- field access ------------------------------------------------- */

TEST(fields_start_end_commit_top)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_NOT_NULL(vs->start, "vs->start is NULL");
    ASSERT_NOT_NULL(vs->end,   "vs->end is NULL");
    ASSERT_TRUE(vs->end > vs->start, "vs->end <= vs->start");
    ASSERT_EQ(vs->commit_top, vs->start, "commit_top != start initially");

    destroy_virt_space(vs);
}

/* ---- reservation size -------------------------------------------- */

TEST(reserved_matches_request)
{
    size_t requested = 2048;
    VirtSpace* vs = create_virt_space(0, requested, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_EQ(vs_reserved(vs), requested, "vs_reserved != requested");
    ASSERT_EQ((size_t)(vs->end - vs->start), requested,
              "pointer diff != requested");

    destroy_virt_space(vs);
}

TEST(initial_committed_is_zero)
{
    VirtSpace* vs = create_virt_space(0, 1024, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_EQ(vs_committed(vs), (size_t)0, "initial committed != 0");
    ASSERT_EQ(vs->commit_top, vs->start, "commit_top != start");

    destroy_virt_space(vs);
}

/* ---- expand ------------------------------------------------------ */

TEST(expand_without_touch)
{
    VirtSpace* vs = create_virt_space(0, 1024, false);
    ASSERT_NOT_NULL(vs, "create failed");

    bool ok = vs_expand(vs, 256, false);
    ASSERT_TRUE(ok, "vs_expand returned false");
    ASSERT_EQ(vs_committed(vs), (size_t)256, "committed != 256");

    destroy_virt_space(vs);
}

TEST(expand_with_touch)
{
    VirtSpace* vs = create_virt_space(0, 256, false);
    ASSERT_NOT_NULL(vs, "create failed");

    /* touch=true: pre-faults every page so the range is physically
     * accessible without on-demand page faults. */
    bool ok = vs_expand(vs, 256, true);
    ASSERT_TRUE(ok, "vs_expand(touch=true) returned false");
    ASSERT_EQ(vs_committed(vs), (size_t)256, "committed != 256");

    /* Write to the entire committed range to verify no SEGV */
    memset((void*)vs->start, 0xAB, 256 * sizeof(HeapWord));

    destroy_virt_space(vs);
}

TEST(expand_multiple_steps)
{
    VirtSpace* vs = create_virt_space(0, 1000, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_TRUE(vs_expand(vs, 300, false), "expand 1");
    ASSERT_EQ(vs_committed(vs), (size_t)300, "committed != 300");

    ASSERT_TRUE(vs_expand(vs, 400, false), "expand 2");
    ASSERT_EQ(vs_committed(vs), (size_t)700, "committed != 700");

    ASSERT_TRUE(vs_expand(vs, 300, false), "expand 3");
    ASSERT_EQ(vs_committed(vs), (size_t)1000, "committed != 1000");

    destroy_virt_space(vs);
}

TEST(expand_beyond_reserved_fails)
{
    VirtSpace* vs = create_virt_space(0, 100, false);
    ASSERT_NOT_NULL(vs, "create failed");

    /* Expand to exact limit */
    ASSERT_TRUE(vs_expand(vs, 100, false), "expand to limit failed");
    ASSERT_EQ(vs_committed(vs), (size_t)100, "committed != 100");

    /* Exceeding must fail */
    bool ok = vs_expand(vs, 1, false);
    ASSERT_TRUE(!ok, "expand beyond reserved should fail");
    ASSERT_EQ(vs_committed(vs), (size_t)100, "committed changed after failed expand");

    destroy_virt_space(vs);
}

/* ---- shrink ------------------------------------------------------ */

TEST(shrink_basic)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    vs_expand(vs, 400, false);
    ASSERT_EQ(vs_committed(vs), (size_t)400, "committed != 400");

    bool ok = vs_shrink(vs, 150);
    ASSERT_TRUE(ok, "vs_shrink returned false");
    ASSERT_EQ(vs_committed(vs), (size_t)250, "committed != 250");

    destroy_virt_space(vs);
}

TEST(shrink_to_zero)
{
    VirtSpace* vs = create_virt_space(0, 128, false);
    ASSERT_NOT_NULL(vs, "create failed");

    vs_expand(vs, 128, false);
    ASSERT_TRUE(vs_shrink(vs, 128), "shrink to 0 failed");
    ASSERT_EQ(vs_committed(vs), (size_t)0,  "committed != 0 after full shrink");
    ASSERT_EQ(vs->commit_top, vs->start, "commit_top != start");

    destroy_virt_space(vs);
}

TEST(shrink_beyond_committed_fails)
{
    VirtSpace* vs = create_virt_space(0, 100, false);
    ASSERT_NOT_NULL(vs, "create failed");

    vs_expand(vs, 50, false);
    bool ok = vs_shrink(vs, 100);
    ASSERT_TRUE(!ok, "shrink beyond committed should fail");
    ASSERT_EQ(vs_committed(vs), (size_t)50, "committed changed after bad shrink");

    destroy_virt_space(vs);
}

/* ---- expand / shrink cycle --------------------------------------- */

TEST(expand_shrink_expand_cycle)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    /* Expand -> shrink -> expand again */
    ASSERT_TRUE(vs_expand(vs, 200, true),  "expand 1");
    ASSERT_TRUE(vs_shrink(vs, 100),         "shrink");
    ASSERT_EQ(vs_committed(vs), (size_t)100, "committed != 100");

    ASSERT_TRUE(vs_expand(vs, 300, true),  "expand 2");
    ASSERT_EQ(vs_committed(vs), (size_t)400, "committed != 400");

    /* Touch the re-expanded range (skip first 50 words, write 300) */
    memset((void*)(vs->start + 50), 0xCD, 300 * sizeof(HeapWord));

    destroy_virt_space(vs);
}

/* ---- NULL safety ------------------------------------------------- */

TEST(destroy_null_is_safe)
{
    destroy_virt_space(NULL);
    /* No crash = pass */
}

/* ---- various sizes ----------------------------------------------- */

TEST(different_sizes)
{
    size_t sizes[] = { 1, 2, 7, 64, 4096, 10000 };
    for (int i = 0; i < 6; i++) {
        VirtSpace* vs = create_virt_space(0, sizes[i], false);
        ASSERT_NOT_NULL(vs, "create failed for size");
        ASSERT_EQ(vs_reserved(vs), sizes[i], "reserved mismatch");
        destroy_virt_space(vs);
    }
}

/* ================================================================== */
/*  Runner                                                            */
/* ================================================================== */

int main(void)
{
    printf("\n=== virt_space tests ===\n\n");

    RUN_TEST(create_destroy_basic);
    RUN_TEST(exec_flag_stored);
    RUN_TEST(noexec_flag_stored);
    RUN_TEST(fields_start_end_commit_top);
    RUN_TEST(reserved_matches_request);
    RUN_TEST(initial_committed_is_zero);
    RUN_TEST(expand_without_touch);
    RUN_TEST(expand_with_touch);
    RUN_TEST(expand_multiple_steps);
    RUN_TEST(expand_beyond_reserved_fails);
    RUN_TEST(shrink_basic);
    RUN_TEST(shrink_to_zero);
    RUN_TEST(shrink_beyond_committed_fails);
    RUN_TEST(expand_shrink_expand_cycle);
    RUN_TEST(destroy_null_is_safe);
    RUN_TEST(different_sizes);

    printf("\n---\n");
    printf("  TOTAL:  %d\n", _tests_run);
    printf("  PASSED: %d\n", _tests_pass);
    printf("  FAILED: %d\n\n", _tests_fail);

    return _tests_fail > 0 ? EXIT_FAILURE : EXIT_SUCCESS;
}
