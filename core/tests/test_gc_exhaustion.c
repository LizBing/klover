#include "gc/gc.h"
#include "gc/gc_heap.h"
#include "memory/comp_space_defs.h"
#include "tests/test_harness.h"

#include <stdlib.h>

TEST(exact_capacity_and_failed_allocation)
{
    /* Separate executable: heap initialization and exhaustion are process local. */
    ASSERT_TRUE(gc_init(4096), "heap initialization failed");
    Klass* klass = (Klass*)(METASPACE_BASE + 8);
    objptr_t first = gcheap_alloc(klass, 4);
    ASSERT_NOT_NULL(first, "first allocation failed");
    ASSERT_EQ((uintptr_t)first, GCHEAP_BASE + sizeof(HeapWord), "null sentinel was not skipped");
    for (size_t i = 0; i < 3; i++) {
        ASSERT_EQ(first->payload[i], (HeapWord)0, "new payload was not zeroed");
        first->payload[i] = (HeapWord)(uintptr_t)(0xCAFE + i);
    }
    size_t remaining = 4096 / sizeof(HeapWord) - 1 - 4;
    ASSERT_NULL(gcheap_alloc(klass, remaining + 1), "oversized request succeeded");
    objptr_t last = gcheap_alloc(klass, remaining);
    ASSERT_NOT_NULL(last, "failed allocation consumed capacity");
    ASSERT_EQ((uintptr_t)last, (uintptr_t)first + 4 * sizeof(HeapWord), "allocation left a gap");
    ASSERT_NULL(gcheap_alloc(klass, 1), "allocation beyond exact capacity succeeded");
    for (size_t i = 0; i < 3; i++) {
        ASSERT_EQ(first->payload[i], (HeapWord)(uintptr_t)(0xCAFE + i), "later allocations damaged an object");
    }
}

int main(void)
{
    RUN_TEST(exact_capacity_and_failed_allocation);
    return _tests_fail ? EXIT_FAILURE : EXIT_SUCCESS;
}
