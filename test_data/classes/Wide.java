public class Wide {
    public static long ladd(long a, long b) { return a + b; }
    public static long lsub(long a, long b) { return a - b; }
    public static long lmul(long a, long b) { return a * b; }
    public static long ldiv(long a, long b) { return a / b; }
    public static long lrem(long a, long b) { return a % b; }
    public static long lneg(long a) { return -a; }

    public static float fadd(float a, float b) { return a + b; }
    public static float fsub(float a, float b) { return a - b; }
    public static float fmul(float a, float b) { return a * b; }
    public static float fdiv(float a, float b) { return a / b; }
    public static float frem(float a, float b) { return a % b; }
    public static float fneg(float a) { return -a; }

    public static double dadd(double a, double b) { return a + b; }
    public static double dsub(double a, double b) { return a - b; }
    public static double dmul(double a, double b) { return a * b; }
    public static double ddiv(double a, double b) { return a / b; }
    public static double drem(double a, double b) { return a % b; }
    public static double dneg(double a) { return -a; }

    // 比较
    public static long lcmp(long a, long b) {
        if (a < b) return -1;
        if (a > b) return 1;
        return 0;
    }
    // fcmp/dcmp 由编译器内联成 fcmpg/fcmpl + if，无法直接测单条指令；
    // 这里用比较结果驱动分支即可覆盖。
    public static int fbranch(float a, float b) {
        if (a > b) return 1;
        if (a < b) return -1;
        return 0;
    }
    public static int dbranch(double a, double b) {
        if (a > b) return 1;
        if (a < b) return -1;
        return 0;
    }

    // 类型转换
    public static long i2l(int a) { return (long) a; }
    public static int l2i(long a) { return (int) a; }
    public static float i2f(int a) { return (float) a; }
    public static double i2d(int a) { return (double) a; }
    public static float l2f(long a) { return (float) a; }
    public static double l2d(long a) { return (double) a; }
    public static int f2i(float a) { return (int) a; }
    public static long f2l(float a) { return (long) a; }
    public static int d2i(double a) { return (int) a; }
    public static long d2l(double a) { return (long) a; }
    public static double f2d(float a) { return (double) a; }
    public static float d2f(double a) { return (float) a; }

    // 位运算
    public static int iand(int a, int b) { return a & b; }
    public static int ior(int a, int b) { return a | b; }
    public static int ixor(int a, int b) { return a ^ b; }
    public static int ishl(int a, int b) { return a << b; }
    public static int ishr(int a, int b) { return a >> b; }
    public static int iushr(int a, int b) { return a >>> b; }

    public static long land(long a, long b) { return a & b; }
    public static long lor(long a, long b) { return a | b; }
    public static long lxor(long a, long b) { return a ^ b; }
    public static long lshl(long a, int b) { return a << b; }
    public static long lshr(long a, int b) { return a >> b; }
    public static long lushr(long a, int b) { return a >>> b; }

    // 自增常数加载（ldc 用不到 int 的话，iconst 系列 + sipush 已覆盖）
    public static int inc5() {
        int x = 0;
        x += 2147483647; // 边界值
        return x;
    }
}
