#include "gc/gc.h"
#include "gc/gc_heap.h"
#include "memory/comp_space_defs.h"
#include "tests/test_harness.h"

#include <pthread.h>
#include <stdlib.h>

enum { WORKERS = 4, ALLOCS = 1000, WORDS = 4 };
static objptr_t objects[WORKERS][ALLOCS];

static void* allocate(void* argument)
{
    size_t worker = (size_t)(uintptr_t)argument;
    for (size_t i = 0; i < ALLOCS; i++) {
        objptr_t obj = gcheap_alloc((Klass*)(METASPACE_BASE + 8), WORDS);
        objects[worker][i] = obj;
        if (obj != NULL) obj->payload[0] = (HeapWord)(uintptr_t)(worker * ALLOCS + i + 1);
    }
    return NULL;
}

static int compare_addresses(const void* left, const void* right)
{
    uintptr_t a = *(const uintptr_t*)left, b = *(const uintptr_t*)right;
    return (a > b) - (a < b);
}

TEST(concurrent_allocations_are_disjoint)
{
    ASSERT_TRUE(gc_init(1024 * 1024), "heap initialization failed");
    pthread_t threads[WORKERS];
    size_t started = 0;
    for (; started < WORKERS; started++) {
        if (pthread_create(&threads[started], NULL, allocate, (void*)(uintptr_t)started) != 0) break;
    }
    for (size_t i = 0; i < started; i++) {
        if (pthread_join(threads[i], NULL) != 0) exit(EXIT_FAILURE);
    }
    ASSERT_EQ(started, (size_t)WORKERS, "could not start all workers");
    uintptr_t addresses[WORKERS * ALLOCS];
    for (size_t worker = 0; worker < WORKERS; worker++) {
        for (size_t i = 0; i < ALLOCS; i++) {
            objptr_t obj = objects[worker][i];
            ASSERT_NOT_NULL(obj, "concurrent allocation failed");
            ASSERT_EQ((uintptr_t)obj % sizeof(HeapWord), (uintptr_t)0, "unaligned object");
            ASSERT_EQ(obj->payload[0], (HeapWord)(uintptr_t)(worker * ALLOCS + i + 1), "payload overwritten");
            addresses[worker * ALLOCS + i] = (uintptr_t)obj;
        }
    }
    qsort(addresses, WORKERS * ALLOCS, sizeof(uintptr_t), compare_addresses);
    for (size_t i = 1; i < WORKERS * ALLOCS; i++) {
        ASSERT_TRUE(addresses[i] - addresses[i - 1] >= WORDS * sizeof(HeapWord), "live allocations overlap");
    }
}

int main(void)
{
    RUN_TEST(concurrent_allocations_are_disjoint);
    return _tests_fail ? EXIT_FAILURE : EXIT_SUCCESS;
}
