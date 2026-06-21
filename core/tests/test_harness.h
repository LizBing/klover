#ifndef TESTS_TEST_HARNESS_H_
#define TESTS_TEST_HARNESS_H_

#include <stdio.h>

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
#define ASSERT_NULL(p, msg) do { if ((p) != NULL) FAIL(msg); } while (0)

#endif /* TESTS_TEST_HARNESS_H_ */
