// Java 8 fixtures. Arguments prevent javac from folding the tested operations.
// Constructors are parsed but never invoked: method invocation is not supported yet.
public class Cases {
    public static int subtract(int left, int right) { return left - right; }
    public static int add(int left, int right) { return left + right; }
    public static long multiply(long left, long right) { return left * right; }
    public static int divide(int left, int right) { return left / right; }
    public static int remainder(int left, int right) { return left % right; }
    public static int shift(int value, int distance) { return value >>> distance; }
    public static long shiftLong(long value, int distance) { return value >>> distance; }
    public static int sum(int count) {
        int result = 0;
        for (int i = 1; i <= count; i++) result += i;
        return result;
    }
    public static int compare(int left, int right) {
        if (left < right) return -1;
        if (left == right) return 0;
        return 1;
    }
    public static int denseSwitch(int key) {
        switch (key) {
            case -1: return 11;
            case 0: return 22;
            case 1: return 33;
            default: return 44;
        }
    }
    public static int sparseSwitch(int key) {
        switch (key) {
            case -1000: return 5;
            case 7: return 6;
            case 1000000: return 7;
            default: return 8;
        }
    }
    public static int toInt(double value) { return (int)value; }
    public static int toByte(int value) { return (byte)value; }
    public static int toChar(int value) { return (char)value; }
    public static float divideFloat(float left, float right) { return left / right; }
    public static double negateDouble(double value) { return -value; }
    public static long mixedWide(int first, long second, double third) { return first + second + (long)third; }
    public static Object reference(Object value) { return value; }
    public static int isNull(Object value) { return value == null ? 1 : 0; }
    public static void nothing() { }
    public static int constantInt() { return 123456789; }
    public static long constantLong() { return -81985529216486895L; }
    public static float constantFloat() { return 1.25f; }
    public static double constantDouble() { return 3.25; }
}
