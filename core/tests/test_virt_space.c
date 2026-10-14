#include "memory/virt_space.h"

#include "tests/test_harness.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

TEST(occupied_fixed_address_preserves_existing_mapping)
{
    size_t size = 2 * get_vm_page_size();
    VirtSpace* original = create_virt_space(0, size, false);
    ASSERT_NOT_NULL(original, "initial reservation failed");
    unsigned char* memory = (unsigned char*)vs_start(original);
    memset(memory, 0xAB, size);

    VirtSpace* collision = create_virt_space(vs_start(original), size, false);
    ASSERT_NULL(collision, "occupied fixed address must fail");
    collision = create_virt_space(vs_start(original) + get_vm_page_size(),
                                  get_vm_page_size(), false);
    ASSERT_NULL(collision, "overlapping fixed address must fail");
    for (size_t i = 0; i < size; i++) {
        ASSERT_EQ(memory[i], (unsigned char)0xAB, "collision modified existing memory");
    }
    ASSERT_EQ(vs_reserved(original), size, "collision changed original reservation");
    destroy_virt_space(original);
}

TEST(available_fixed_address_is_exact)
{
    size_t size = get_vm_page_size();
    VirtSpace* probe = create_virt_space(0, size, false);
    ASSERT_NOT_NULL(probe, "initial reservation failed");
    uintptr_t base = vs_start(probe);
    destroy_virt_space(probe);

    VirtSpace* exact = create_virt_space(base, size, false);
    ASSERT_NOT_NULL(exact, "available fixed address was rejected");
    ASSERT_EQ(vs_start(exact), base, "fixed address reservation moved elsewhere");
    memset((void*)base, 0xCD, size);
    ASSERT_EQ(((unsigned char*)base)[size - 1], (unsigned char)0xCD,
              "fixed address reservation is not writable");
    destroy_virt_space(exact);
}

TEST(unaligned_fixed_address_fails)
{
    ASSERT_NULL(create_virt_space(1, get_vm_page_size(), false),
                "unaligned fixed address must fail");
}

TEST(overflowing_fixed_address_range_fails)
{
    size_t page_size = get_vm_page_size();
    uintptr_t base = UINTPTR_MAX - (page_size - 1);
    ASSERT_NULL(create_virt_space(base, 2 * page_size, false),
                "overflowing fixed address range must fail");
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
    ASSERT_TRUE(vs_exec(vs), "expected vs_exec(vs) == true");
    destroy_virt_space(vs);
}

TEST(noexec_flag_stored)
{
    VirtSpace* vs = create_virt_space(0, 256, false);
    ASSERT_NOT_NULL(vs, "create_virt_space(exec=false) returned NULL");
    ASSERT_TRUE(!vs_exec(vs), "expected vs_exec(vs) == false");
    destroy_virt_space(vs);
}

/* ---- field access ------------------------------------------------- */

TEST(fields_start_end_commit_top)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_TRUE(vs_start(vs) != 0, "vs_start(vs) is zero");
    ASSERT_TRUE(vs_end(vs) != 0, "vs_end(vs) is zero");
    ASSERT_EQ(vs_start(vs) % get_vm_page_size(), (uintptr_t)0,
              "vs_start(vs) is not page aligned");
    ASSERT_TRUE(vs_end(vs) > vs_start(vs), "vs_end(vs) <= vs_start(vs)");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs), "commit_top != start initially");

    destroy_virt_space(vs);
}

/* ---- reservation size -------------------------------------------- */

TEST(reserved_matches_page_aligned_request)
{
    size_t requested = 2 * get_vm_page_size();
    VirtSpace* vs = create_virt_space(0, requested, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_EQ(vs_reserved(vs), requested, "vs_reserved != requested");
    ASSERT_EQ((size_t)(vs_end(vs) - vs_start(vs)), requested,
              "address difference != requested bytes");

    destroy_virt_space(vs);
}

TEST(zero_size_reserves_one_page)
{
    VirtSpace* vs = create_virt_space(0, 0, false);
    ASSERT_NOT_NULL(vs, "create failed for zero size");

    ASSERT_EQ(vs_reserved(vs), get_vm_page_size(), "zero size must reserve one page");
    ASSERT_EQ(vs_committed(vs), (size_t)0, "initial committed != 0");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs), "commit_top != start");

    destroy_virt_space(vs);
}

TEST(overflowing_reservation_fails)
{
    VirtSpace* vs = create_virt_space(0, SIZE_MAX, false);
    ASSERT_NULL(vs, "page rounding overflow must fail");
}

TEST(initial_committed_is_zero)
{
    VirtSpace* vs = create_virt_space(0, 1024, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_EQ(vs_committed(vs), (size_t)0, "initial committed != 0");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs), "commit_top != start");

    destroy_virt_space(vs);
}

/* ---- expand ------------------------------------------------------ */

TEST(expand_without_touch)
{
    VirtSpace* vs = create_virt_space(0, 1024, false);
    ASSERT_NOT_NULL(vs, "create failed");

    uintptr_t start = vs_expand_by(vs, 256, false);
    ASSERT_EQ(start, vs_start(vs), "expand must return the old commit top");
    ASSERT_EQ(vs_committed(vs), (size_t)256, "committed != 256");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs) + 256, "commit_top did not advance by 256 bytes");

    destroy_virt_space(vs);
}

TEST(expand_with_touch)
{
    size_t committed_bytes = 2 * get_vm_page_size() + 17;
    VirtSpace* vs = create_virt_space(0, committed_bytes, false);
    ASSERT_NOT_NULL(vs, "create failed");

    /* touch=true: pre-faults every page so the range is physically
     * accessible without on-demand page faults. */
    uintptr_t start = vs_expand_by(vs, committed_bytes, true);
    ASSERT_EQ(start, vs_start(vs), "expand must return the old commit top");
    ASSERT_EQ(vs_committed(vs), committed_bytes, "committed byte count mismatch");

    /* Write to the entire committed range to verify no SEGV */
    memset((void*)vs_start(vs), 0xAB, committed_bytes);
    ASSERT_EQ(((unsigned char*)vs_start(vs))[committed_bytes - 1], 0xAB,
              "last committed byte is not writable");

    destroy_virt_space(vs);
}

TEST(expand_multiple_steps)
{
    VirtSpace* vs = create_virt_space(0, 1000, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_EQ(vs_expand_by(vs, 300, false), vs_start(vs), "expand 1 returned wrong address");
    ASSERT_EQ(vs_committed(vs), (size_t)300, "committed != 300");

    ASSERT_EQ(vs_expand_by(vs, 400, false), vs_start(vs) + 300, "expand 2 returned wrong address");
    ASSERT_EQ(vs_committed(vs), (size_t)700, "committed != 700");

    ASSERT_EQ(vs_expand_by(vs, 300, false), vs_start(vs) + 700, "expand 3 returned wrong address");
    ASSERT_EQ(vs_committed(vs), (size_t)1000, "committed != 1000");

    destroy_virt_space(vs);
}

TEST(touch_unaligned_range_reaches_each_page)
{
    size_t page_size = get_vm_page_size();
    VirtSpace* vs = create_virt_space(0, 2 * page_size, false);
    ASSERT_NOT_NULL(vs, "create failed");
    unsigned char* memory = (unsigned char*)vs_start(vs);
    memset(memory, 0xAB, 2 * page_size);

    ASSERT_EQ(vs_expand_by(vs, 1, false), vs_start(vs), "initial expand failed");
    ASSERT_EQ(vs_expand_by(vs, page_size, true), vs_start(vs) + 1,
              "unaligned expand returned wrong address");
    ASSERT_EQ(memory[0], 0xAB, "touch overwrote previously committed data");
    ASSERT_EQ(memory[1], 0, "first page was not touched");
    ASSERT_EQ(memory[page_size], 0, "last page was not touched");
    ASSERT_EQ(memory[page_size + 1], 0xAB, "touch wrote beyond the committed range");
    destroy_virt_space(vs);
}

TEST(expand_beyond_reserved_fails)
{
    VirtSpace* vs = create_virt_space(0, 100, false);
    ASSERT_NOT_NULL(vs, "create failed");

    size_t reserved = vs_reserved(vs);
    ASSERT_TRUE(vs_expand_by(vs, 100, false), "initial expand failed");
    ASSERT_EQ(vs_committed(vs), (size_t)100, "committed != 100");

    ASSERT_TRUE(!vs_expand_by(vs, reserved - 100 + 1, false),
                "expand beyond remaining capacity should fail");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs) + 100, "commit_top changed after failed expand");

    /* Expand to the page-rounded reservation limit. */
    ASSERT_TRUE(vs_expand_by(vs, reserved - 100, false), "expand to limit failed");
    ASSERT_EQ(vs_committed(vs), reserved, "committed != reserved");

    /* Exceeding must fail */
    uintptr_t start = vs_expand_by(vs, 1, false);
    ASSERT_EQ(start, (uintptr_t)0, "expand beyond reserved should fail");
    ASSERT_EQ(vs_committed(vs), reserved, "committed changed after failed expand");
    ASSERT_EQ(vs_commit_top(vs), vs_end(vs), "commit_top changed after failed expand");

    destroy_virt_space(vs);
}

/* ---- shrink ------------------------------------------------------ */

TEST(shrink_basic)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_TRUE(vs_expand_by(vs, 400, false), "initial expand failed");
    ASSERT_EQ(vs_committed(vs), (size_t)400, "committed != 400");

    uintptr_t top = vs_shrink_by(vs, 150);
    ASSERT_EQ(top, vs_start(vs) + 250, "shrink must return the new commit top");
    ASSERT_EQ(vs_committed(vs), (size_t)250, "committed != 250");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs) + 250, "commit_top did not shrink by 150 bytes");

    destroy_virt_space(vs);
}

TEST(shrink_to_zero)
{
    VirtSpace* vs = create_virt_space(0, 128, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_TRUE(vs_expand_by(vs, 128, false), "initial expand failed");
    ASSERT_EQ(vs_shrink_by(vs, 128), vs_start(vs), "full shrink must return start");
    ASSERT_EQ(vs_committed(vs), (size_t)0,  "committed != 0 after full shrink");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs), "commit_top != start");

    destroy_virt_space(vs);
}

TEST(shrink_beyond_committed_fails)
{
    VirtSpace* vs = create_virt_space(0, 100, false);
    ASSERT_NOT_NULL(vs, "create failed");

    ASSERT_TRUE(vs_expand_by(vs, 50, false), "initial expand failed");
    uintptr_t top = vs_shrink_by(vs, 100);
    ASSERT_EQ(top, (uintptr_t)0, "shrink beyond committed should fail");
    ASSERT_EQ(vs_committed(vs), (size_t)50, "committed changed after bad shrink");
    ASSERT_EQ(vs_commit_top(vs), vs_start(vs) + 50, "commit_top changed after bad shrink");

    destroy_virt_space(vs);
}

/* ---- expand / shrink cycle --------------------------------------- */

TEST(expand_shrink_expand_cycle)
{
    VirtSpace* vs = create_virt_space(0, 512, false);
    ASSERT_NOT_NULL(vs, "create failed");

    /* Expand -> shrink -> expand again */
    ASSERT_EQ(vs_expand_by(vs, 200, true), vs_start(vs), "expand 1 returned wrong address");
    ASSERT_EQ(vs_shrink_by(vs, 100), vs_start(vs) + 100, "shrink returned wrong address");
    ASSERT_EQ(vs_committed(vs), (size_t)100, "committed != 100");

    ASSERT_EQ(vs_expand_by(vs, 300, true), vs_start(vs) + 100, "expand 2 returned wrong address");
    ASSERT_EQ(vs_committed(vs), (size_t)400, "committed != 400");

    /* Write the 300 bytes committed by the second expansion. */
    memset((void*)(vs_start(vs) + 100), 0xCD, 300);

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
    size_t page_size = get_vm_page_size();
    size_t sizes[] = { 1, 2, 7, 64, page_size - 1, page_size,
                       page_size + 1, 2 * page_size, 2 * page_size + 1 };
    size_t expected[] = { page_size, page_size, page_size, page_size,
                          page_size, page_size, 2 * page_size,
                          2 * page_size, 3 * page_size };
    for (size_t i = 0; i < sizeof(sizes) / sizeof(sizes[0]); i++) {
        VirtSpace* vs = create_virt_space(0, sizes[i], false);
        ASSERT_NOT_NULL(vs, "create failed for size");
        ASSERT_EQ(vs_reserved(vs), expected[i], "page-rounded reserved size mismatch");
        ASSERT_EQ(vs_end(vs) - vs_start(vs), expected[i], "address difference mismatch");
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
    RUN_TEST(occupied_fixed_address_preserves_existing_mapping);
    RUN_TEST(available_fixed_address_is_exact);
    RUN_TEST(unaligned_fixed_address_fails);
    RUN_TEST(overflowing_fixed_address_range_fails);
    RUN_TEST(exec_flag_stored);
    RUN_TEST(noexec_flag_stored);
    RUN_TEST(fields_start_end_commit_top);
    RUN_TEST(reserved_matches_page_aligned_request);
    RUN_TEST(zero_size_reserves_one_page);
    RUN_TEST(overflowing_reservation_fails);
    RUN_TEST(initial_committed_is_zero);
    RUN_TEST(expand_without_touch);
    RUN_TEST(expand_with_touch);
    RUN_TEST(expand_multiple_steps);
    RUN_TEST(touch_unaligned_range_reaches_each_page);
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
