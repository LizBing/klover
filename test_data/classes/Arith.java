public class Arith {
    public static int mul2(int x) {
        int y = x;
        y = y * 2;
        return y;
    }

    public static int neg(int x) {
        return -x;
    }

    public static int sub(int a, int b) {
        return a - b;
    }

    public static int rem(int a, int b) {
        return a % b;
    }

    public static int addConst() {
        int x = 0;
        x += 5;
        x += 10;
        return x;
    }

    public static int pushByte() {
        return 100;
    }

    public static int pushShort() {
        return 10000;
    }
}
