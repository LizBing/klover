public class ArithmeticOps {
    public static int iadd(int a, int b) { return a + b; }
    public static long ladd(long a, long b) { return a + b; }
    public static float fadd(float a, float b) { return a + b; }
    public static double dadd(double a, double b) { return a + b; }

    public static int isub(int a, int b) { return a - b; }
    public static long lsub(long a, long b) { return a - b; }
    public static float fsub(float a, float b) { return a - b; }
    public static double dsub(double a, double b) { return a - b; }

    public static int imul(int a, int b) { return a * b; }
    public static long lmul(long a, long b) { return a * b; }
    public static float fmul(float a, float b) { return a * b; }
    public static double dmul(double a, double b) { return a * b; }

    public static int idiv(int a, int b) { return a / b; }
    public static long ldiv(long a, long b) { return a / b; }
    public static float fdiv(float a, float b) { return a / b; }
    public static double ddiv(double a, double b) { return a / b; }

    public static int irem(int a, int b) { return a % b; }
    public static long lrem(long a, long b) { return a % b; }
    public static float frem(float a, float b) { return a % b; }
    public static double drem(double a, double b) { return a % b; }

    public static int ineg(int value) { return -value; }
    public static long lneg(long value) { return -value; }
    public static float fneg(float value) { return -value; }
    public static double dneg(double value) { return -value; }

    public static int ishl(int value, int distance) { return value << distance; }
    public static long lshl(long value, int distance) { return value << distance; }
    public static int ishr(int value, int distance) { return value >> distance; }
    public static long lshr(long value, int distance) { return value >> distance; }
    public static int iushr(int value, int distance) { return value >>> distance; }
    public static long lushr(long value, int distance) { return value >>> distance; }

    public static int iand(int a, int b) { return a & b; }
    public static long land(long a, long b) { return a & b; }
    public static int ior(int a, int b) { return a | b; }
    public static long lor(long a, long b) { return a | b; }
    public static int ixor(int a, int b) { return a ^ b; }
    public static long lxor(long a, long b) { return a ^ b; }

    public static int iinc(int value) {
        value += 7;
        return value;
    }
}
