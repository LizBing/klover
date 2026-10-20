#include "memory/virt_space.h"
#include "tests/test_harness.h"

#include <stdint.h>
#include <stdlib.h>

TEST(reservation_and_accounting)
{
    size_t page = get_vm_page_size();
    VirtSpace* vs = create_virt_space(0, 2 * page + 1, false);
    ASSERT_NOT_NULL(vs, "reservation failed");
    ASSERT_EQ(vs_reserved(vs), 3 * page, "reservation must round to pages");
    ASSERT_EQ(vs_committed(vs), (size_t)0, "new space is uncommitted");
    uintptr_t start = vs_start(vs);
    ASSERT_EQ(start % page, (uintptr_t)0, "mapping is not page aligned");
    ASSERT_TRUE(!vs_exec(vs), "unexpected execute flag");

    ASSERT_EQ(vs_expand_by(vs, page + 1, true), start, "expansion returned wrong old top");
    ASSERT_EQ(vs_committed(vs), page + 1, "expansion accounting is wrong");
    ((unsigned char*)start)[page] = 0xAB;
    ASSERT_EQ(vs_expand_by(vs, 2 * page, false), (uintptr_t)0, "over-expansion succeeded");
    ASSERT_EQ(vs_committed(vs), page + 1, "failed expansion changed state");
    ASSERT_EQ(vs_shrink_by(vs, page + 2), (uintptr_t)0, "over-shrink succeeded");
    ASSERT_EQ(vs_committed(vs), page + 1, "failed shrink changed state");
    ASSERT_EQ(((unsigned char*)start)[page], 0xAB, "failed operation damaged memory");

    /* Restore page alignment before asking madvise to release whole pages. */
    ASSERT_EQ(vs_expand_by(vs, page - 1, true), start + page + 1, "second expansion failed");
    ASSERT_EQ(vs_shrink_by(vs, page), start + page, "shrink returned wrong top");
    ASSERT_EQ(vs_committed(vs), page, "shrink accounting is wrong");
    ASSERT_EQ(vs_expand_by(vs, 2 * page, true), start + page, "capacity was not reusable");
    ASSERT_EQ(vs_commit_top(vs), vs_end(vs), "full expansion did not reach reserved end");
    destroy_virt_space(vs);
}

TEST(default_size_and_invalid_reservations)
{
    VirtSpace* vs = create_virt_space(0, 0, true);
    ASSERT_NOT_NULL(vs, "zero size should reserve one page");
    ASSERT_EQ(vs_reserved(vs), get_vm_page_size(), "wrong default reservation");
    ASSERT_TRUE(vs_exec(vs), "execute intent was not stored");
    destroy_virt_space(vs);
    destroy_virt_space(NULL);
    ASSERT_NULL(create_virt_space(1, 4096, false), "unaligned base was accepted");
    ASSERT_NULL(create_virt_space(0, SIZE_MAX, false), "rounding overflow was accepted");
}

TEST(occupied_address_is_not_replaced)
{
    size_t page = get_vm_page_size();
    VirtSpace* owner = create_virt_space(0, page, false);
    ASSERT_NOT_NULL(owner, "initial mapping failed");
    uintptr_t start = vs_start(owner);
    *(unsigned char*)start = 0x5A;
    VirtSpace* other = create_virt_space(start, page, false);
    ASSERT_NULL(other, "occupied exact address was accepted");
    ASSERT_EQ(*(unsigned char*)start, 0x5A, "reservation replaced an existing mapping");
    destroy_virt_space(owner);
}

int main(void)
{
    RUN_TEST(reservation_and_accounting);
    RUN_TEST(default_size_and_invalid_reservations);
    RUN_TEST(occupied_address_is_not_replaced);
    return _tests_fail ? EXIT_FAILURE : EXIT_SUCCESS;
}
