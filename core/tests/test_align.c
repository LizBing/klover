#include "utils/align.h"
#include "tests/test_harness.h"

#include <stdint.h>
#include <stdlib.h>

TEST(expression_arguments)
{
    ASSERT_EQ(align_down(7, 1 << 3), (size_t)0, "shift alignment rounded down incorrectly");
    ASSERT_EQ(align_up(7, 1 << 3), (size_t)8, "shift alignment rounded up incorrectly");
    ASSERT_EQ(align_down(8 + 7, 1 << 3), (size_t)8, "value expression rounded down incorrectly");
    ASSERT_EQ(align_up(8 + 7, 1 << 3), (size_t)16, "value expression rounded up incorrectly");
    ASSERT_TRUE(is_aligned(4 + 4, 1 << 3), "aligned expression was rejected");
}

TEST(arguments_are_evaluated_once)
{
    size_t n = 7;
    size_t alignment = 8;
    size_t result = align_up(n++, alignment++);
    ASSERT_EQ(result, (size_t)8, "round up returned wrong result");
    ASSERT_EQ(n, (size_t)8, "round up evaluated value more than once");
    ASSERT_EQ(alignment, (size_t)9, "round up evaluated alignment more than once");

    n = 7;
    alignment = 8;
    result = align_down(n++, alignment++);
    ASSERT_EQ(result, (size_t)0, "round down returned wrong result");
    ASSERT_EQ(n, (size_t)8, "round down evaluated value more than once");
    ASSERT_EQ(alignment, (size_t)9, "round down evaluated alignment more than once");

    n = 8;
    alignment = 8;
    bool aligned = is_aligned(n++, alignment++);
    ASSERT_TRUE(aligned, "alignment check returned wrong result");
    ASSERT_EQ(n, (size_t)9, "alignment check evaluated value more than once");
    ASSERT_EQ(alignment, (size_t)9, "alignment check evaluated alignment more than once");
}

TEST(boundary_values)
{
    for (size_t alignment = 1; alignment <= 32768; alignment *= 2) {
        ASSERT_EQ(align_up(0, alignment), (size_t)0, "zero rounded up incorrectly");
        ASSERT_EQ(align_down(0, alignment), (size_t)0, "zero rounded down incorrectly");
        ASSERT_EQ(align_up(alignment, alignment), alignment, "aligned value changed");
        ASSERT_EQ(align_down(alignment, alignment), alignment, "aligned value changed");
        ASSERT_EQ(align_up(alignment + 1, alignment),
                  alignment == 1 ? alignment + 1 : 2 * alignment,
                  "value above boundary rounded up incorrectly");
        ASSERT_TRUE(!is_aligned(alignment + 1, alignment) || alignment == 1,
                    "unaligned value was accepted");
        size_t highest_aligned = SIZE_MAX - (alignment - 1);
        ASSERT_EQ(align_up(highest_aligned, alignment), highest_aligned,
                  "highest aligned value changed");
        ASSERT_EQ(align_down(SIZE_MAX, alignment), highest_aligned,
                  "maximum value rounded down incorrectly");
    }
}

int main(void)
{
    RUN_TEST(expression_arguments);
    RUN_TEST(arguments_are_evaluated_once);
    RUN_TEST(boundary_values);
    return _tests_fail > 0 ? EXIT_FAILURE : EXIT_SUCCESS;
}
