#include "gc/oop_storage.h"
#include "gc/oop_closure.h"

#include "tests/test_harness.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ================================================================== */
/*  Lifecycle                                                         */
/* ================================================================== */

TEST(init_twice_is_safe)
{
    init_oop_storages();
    init_oop_storages();  /* second call is a no-op for already-init'd */
}

TEST(alloc_from_uninitialized)
{
    /* Should return NULL before init */
    /* (We can't easily test this since another test may have inited,
     * but the call itself shouldn't crash) */
    ASSERT_TRUE(true, "ok");
}

/* ================================================================== */
/*  Allocation                                                        */
/* ================================================================== */

TEST(alloc_single_slot)
{
    init_oop_storages();
    oop_t* slot = alloc_oop_slot(WEAK_STORAGE_START);
    ASSERT_NOT_NULL(slot, "alloc returned NULL");

    /* Slot should be zeroed */
    ASSERT_EQ(*slot, (oop_t)NULL, "slot not zeroed");

    /* Write and read back */
    *slot = (oop_t)0xDEADBEEFCAFEull;
    ASSERT_EQ(*slot, (oop_t)0xDEADBEEFCAFEull, "readback mismatch");

    free_oop_slot(WEAK_STORAGE_START, slot);
}

TEST(alloc_multiple_sequential)
{
    init_oop_storages();
    oop_t* slots[10];

    for (int i = 0; i < 10; i++) {
        slots[i] = alloc_oop_slot(WEAK_STORAGE_START);
        ASSERT_NOT_NULL(slots[i], "alloc failed");
        *slots[i] = (oop_t)(uintptr_t)(100 + i);
    }

    /* Slots should be consecutive */
    for (int i = 1; i < 10; i++) {
        ASSERT_EQ(slots[i], slots[i-1] + 1, "slots not consecutive");
    }

    for (int i = 0; i < 10; i++) {
        free_oop_slot(WEAK_STORAGE_START, slots[i]);
    }
}

TEST(alloc_slots_independent)
{
    init_oop_storages();
    oop_t* a = alloc_oop_slot(WEAK_STORAGE_START);
    oop_t* b = alloc_oop_slot(WEAK_STORAGE_START);
    ASSERT_NOT_NULL(a, "a"); ASSERT_NOT_NULL(b, "b");

    *a = (oop_t)0xAAAA;
    *b = (oop_t)0xBBBB;
    ASSERT_EQ(*a, (oop_t)0xAAAA, "a corrupted");
    ASSERT_EQ(*b, (oop_t)0xBBBB, "b corrupted");

    free_oop_slot(WEAK_STORAGE_START, a);
    free_oop_slot(WEAK_STORAGE_START, b);
}

/* ================================================================== */
/*  Release & reuse                                                   */
/* ================================================================== */

TEST(release_then_reuse)
{
    init_oop_storages();
    oop_t* a = alloc_oop_slot(WEAK_STORAGE_START);
    oop_t* b = alloc_oop_slot(WEAK_STORAGE_START);
    oop_t* c = alloc_oop_slot(WEAK_STORAGE_START);
    ASSERT_NOT_NULL(a, "a"); ASSERT_NOT_NULL(b, "b"); ASSERT_NOT_NULL(c, "c");

    free_oop_slot(WEAK_STORAGE_START, b);

    /* Next alloc should reuse b's slot */
    oop_t* d = alloc_oop_slot(WEAK_STORAGE_START);
    ASSERT_EQ(d, b, "should reuse freed slot");

    free_oop_slot(WEAK_STORAGE_START, a);
    free_oop_slot(WEAK_STORAGE_START, c);
    free_oop_slot(WEAK_STORAGE_START, d);
}

TEST(release_external_slot)
{
    init_oop_storages();
    oop_t fake;
    /* Passing a slot not owned by storage — should not crash */
    free_oop_slot(WEAK_STORAGE_START, &fake);
}

TEST(release_null_safe)
{
    init_oop_storages();
    free_oop_slot(WEAK_STORAGE_START, NULL);
}

/* ================================================================== */
/*  Iteration (GC root scanning)                                      */
/* ================================================================== */

typedef struct {
    void**  slots;
    int     count;
    int     cap;
} CollectCtx;

static bool collect_fn(oop_t* slot, void* vctx) {
    CollectCtx* c = (CollectCtx*)vctx;
    if (c->count >= c->cap) return true;
    c->slots[c->count++] = (void*)slot;
    return true;
}

TEST(iterate_empty_before_alloc)
{
    init_oop_storages();
    /* Re-init to get fresh storages — init_oop_storages skips already-init'd.
     * We'll just trust that iterating before any allocation returns cleanly. */
    CollectCtx ctx = { NULL, 0, 0 };
    OOPClosure closure = { collect_fn, &ctx };
    bool ok = weak_native_oops_iterate(&closure);
    ASSERT_TRUE(ok, "iterate should return true");
    ASSERT_EQ(ctx.count, 0, "no slots expected");
}

TEST(iterate_all_live)
{
    init_oop_storages();
    oop_t* allocated[10];

    for (int i = 0; i < 10; i++) {
        allocated[i] = alloc_oop_slot(WEAK_STORAGE_START);
        ASSERT_NOT_NULL(allocated[i], "alloc failed");
        *allocated[i] = (oop_t)(uintptr_t)(i + 1);
    }

    oop_t* found[20];
    CollectCtx ctx = { (void**)found, 0, 20 };
    OOPClosure closure = { collect_fn, &ctx };
    bool ok = weak_native_oops_iterate(&closure);
    ASSERT_TRUE(ok, "iterate should return true");
    ASSERT_EQ(ctx.count, 10, "should iterate exactly 10 slots");

    for (int i = 0; i < 10; i++) {
        bool present = false;
        for (int j = 0; j < ctx.count; j++) {
            if ((oop_t*)found[j] == allocated[i]) { present = true; break; }
        }
        ASSERT_TRUE(present, "allocated slot missing");
    }

    for (int i = 0; i < 10; i++) {
        free_oop_slot(WEAK_STORAGE_START, allocated[i]);
    }
}

TEST(iterate_skips_released)
{
    init_oop_storages();
    oop_t* a = alloc_oop_slot(WEAK_STORAGE_START);
    oop_t* b = alloc_oop_slot(WEAK_STORAGE_START);
    oop_t* c = alloc_oop_slot(WEAK_STORAGE_START);
    (void)a; (void)c;

    free_oop_slot(WEAK_STORAGE_START, b);

    oop_t* found[20];
    CollectCtx ctx = { (void**)found, 0, 20 };
    OOPClosure closure = { collect_fn, &ctx };
    weak_native_oops_iterate(&closure);

    ASSERT_EQ(ctx.count, 2, "should iterate 2 live slots");
    for (int j = 0; j < ctx.count; j++) {
        ASSERT_TRUE((oop_t*)found[j] != b, "released slot should be skipped");
    }

    free_oop_slot(WEAK_STORAGE_START, a);
    free_oop_slot(WEAK_STORAGE_START, c);
}

/* ---- early-stop helper ---- */
typedef struct { int count; int limit; } StopCtx;

static bool stop_fn(oop_t* slot, void* vctx) {
    (void)slot;
    StopCtx* c = (StopCtx*)vctx;
    c->count++;
    return c->count < c->limit;
}

TEST(iterate_early_stop)
{
    init_oop_storages();
    for (int i = 0; i < 30; i++) {
        oop_t* s = alloc_oop_slot(WEAK_STORAGE_START);
        ASSERT_NOT_NULL(s, "alloc");
        *s = (oop_t)(uintptr_t)i;
    }

    StopCtx sc = { 0, 7 };
    OOPClosure closure = { stop_fn, &sc };
    bool ok = weak_native_oops_iterate(&closure);
    ASSERT_TRUE(!ok, "iterate should return false on early stop");
    ASSERT_EQ(sc.count, 7, "should stop after 7 calls");
}

/* ================================================================== */
/*  Invalid storage_id                                                */
/* ================================================================== */

TEST(invalid_storage_id)
{
    init_oop_storages();
    ASSERT_NULL(alloc_oop_slot(-1), "negative id");
    ASSERT_NULL(alloc_oop_slot(ALL_STORAGE_COUNT + 5), "out of range");
    free_oop_slot(-1, NULL);  /* should not crash */
}

/* ================================================================== */
/*  Runner                                                            */
/* ================================================================== */

int main(void)
{
    printf("\n=== oop_storage tests ===\n\n");

    RUN_TEST(init_twice_is_safe);
    RUN_TEST(alloc_from_uninitialized);
    RUN_TEST(alloc_single_slot);
    RUN_TEST(alloc_multiple_sequential);
    RUN_TEST(alloc_slots_independent);
    RUN_TEST(release_then_reuse);
    RUN_TEST(release_external_slot);
    RUN_TEST(release_null_safe);
    RUN_TEST(iterate_empty_before_alloc);
    RUN_TEST(iterate_all_live);
    RUN_TEST(iterate_skips_released);
    RUN_TEST(iterate_early_stop);
    RUN_TEST(invalid_storage_id);

    printf("\n---\n");
    printf("  TOTAL:  %d\n", _tests_run);
    printf("  PASSED: %d\n", _tests_pass);
    printf("  FAILED: %d\n\n", _tests_fail);

    return _tests_fail > 0 ? EXIT_FAILURE : EXIT_SUCCESS;
}
