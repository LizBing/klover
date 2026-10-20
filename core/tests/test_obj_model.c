#include "obj_model/markword.h"
#include "tests/test_harness.h"

#include <stdlib.h>

TEST(compressed_pointer_boundaries)
{
    ASSERT_EQ(comp_ptr_encode(METASPACE_BASE, NULL), (comp_ptr_t)0, "null encoding");
    ASSERT_NULL(comp_ptr_decode(METASPACE_BASE, 0), "null decoding");
    comp_ptr_t values[] = { 1, 2, 12345, UINT32_MAX };
    for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); i++) {
        uintptr_t address = METASPACE_BASE + ((uintptr_t)values[i] << 3);
        ASSERT_EQ((uintptr_t)comp_ptr_decode(METASPACE_BASE, values[i]), address,
                  "compressed pointer decoded to wrong address");
        ASSERT_EQ(comp_ptr_encode(METASPACE_BASE, (void*)address), values[i],
                  "valid compressed pointer did not round trip");
    }
}

TEST(markword_fields_are_independent)
{
    comp_ptr_t values[] = { 0, 1, 0x87654321u, UINT32_MAX };
    for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); i++) {
        uint64_t word = mw_default(values[i]);
        ASSERT_EQ(mw_read_lock_value(word), LOCKVALUE_NONE, "default lock bits");
        ASSERT_EQ(mw_read_klass_comp_ptr(word), values[i], "klass bits lost");
        for (int lock = 0; lock < 4; lock++) {
            uint64_t changed = mw_write_lock_value(word, lock);
            ASSERT_EQ(mw_read_lock_value(changed), lock, "lock write failed");
            ASSERT_EQ(mw_read_klass_comp_ptr(changed), values[i], "lock write changed klass");
            changed = mw_write_klass_comp_ptr(changed, ~values[i]);
            ASSERT_EQ(mw_read_lock_value(changed), lock, "klass write changed lock");
            ASSERT_EQ(mw_read_klass_comp_ptr(changed), (comp_ptr_t)~values[i], "klass write failed");
        }
    }
}

int main(void)
{
    RUN_TEST(compressed_pointer_boundaries);
    RUN_TEST(markword_fields_are_independent);
    return _tests_fail ? EXIT_FAILURE : EXIT_SUCCESS;
}
