#include "tests/test_harness.h"
#include <stdlib.h>

TEST(intentional_failure) { FAIL("intentional harness self-test"); }
TEST(success) { ASSERT_EQ(2 + 2, 4, "arithmetic"); }

int main(void)
{
    RUN_TEST(intentional_failure);
    RUN_TEST(success);
    /* This executable succeeds only if failure accounting is correct. */
    return (_tests_run == 2 && _tests_pass == 1 && _tests_fail == 1)
        ? EXIT_SUCCESS : EXIT_FAILURE;
}
