public class ConstantOps {
    public static Object nullConstant() { return null; }

    public static int intMinusOne() { return -1; }
    public static int intZero() { return 0; }
    public static int intOne() { return 1; }
    public static int intTwo() { return 2; }
    public static int intThree() { return 3; }
    public static int intFour() { return 4; }
    public static int intFive() { return 5; }

    public static long longZero() { return 0L; }
    public static long longOne() { return 1L; }

    public static float floatZero() { return 0.0f; }
    public static float floatOne() { return 1.0f; }
    public static float floatTwo() { return 2.0f; }

    public static double doubleZero() { return 0.0d; }
    public static double doubleOne() { return 1.0d; }

    public static int byteImmediate() { return -100; }
    public static int shortImmediate() { return -30000; }
    public static int intPoolConstant() { return 100000; }
    public static float floatPoolConstant() { return 3.25f; }
    public static long longPoolConstant() { return 1234567890123L; }
    public static double doublePoolConstant() { return 3.25d; }
}
