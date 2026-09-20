// Fixtures restricted to the instructions currently supported by the interpreter.
public class InstructionOps {
    public static int longCompare(long left, long right) {
        if (left < right) return -1;
        if (left > right) return 1;
        return 0;
    }
    public static int scratch(int a, int b, int c, int d, int e, int f) { return a; }
    public static long longStore(long value) { long local = value; return local; }
    public static float floatStore(float value) { float local = value; return local; }
    public static double doubleStore(double value) { double local = value; return local; }
    public static int narrowByte(int value) { return (byte) value; }
    public static int narrowChar(int value) { return (char) value; }
    public static int narrowShort(int value) { return (short) value; }
    public static int wideIncrement(int value) { value += 1000; return value; }
    public static int negativeIncrement(int value) { value -= 1000; return value; }
    public static boolean isNull(Object value) { return value == null; }
    public static boolean isNonNull(Object value) { return value != null; }
    public static boolean same(Object left, Object right) { return left == right; }
    public static boolean different(Object left, Object right) { return left != right; }
}
