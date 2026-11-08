// All executed paths use only the supported interpreter subset.
public class StaticCalls {
    private static int subtract(int a, int b) { return a - b; }
    public static int sameClass(int a, int b) { return 100 + subtract(a, b); }
    public static int repeated(int a, int b) { return subtract(a, b) - subtract(b, a); }
    public static int crossClass(int a, int b) { return StaticCallee.combine(a, b); }
    public static int inherited(int value) { return StaticLeaf.inherited(value); }
    public static int interfaceCall(int value) { return StaticOperation.apply(value); }
    public static int overloaded(int value) { return StaticCallee.identity(value); }
    public static long overloaded(long value) { return StaticCallee.identity(value); }
    public static long mixed(int a, long b, double c, float d, Object e) {
        return 100L + StaticCallee.mixed(a, b, c, d, e);
    }
    public static float floatCall(float value) { return StaticCallee.identity(value); }
    public static double doubleCall(double value) { return StaticCallee.identity(value); }
    public static Object referenceCall(Object value) { return StaticCallee.reference(value); }
    public static int[] arrayCall(int[] value) { return StaticCallee.array(value); }
    public static int narrow(int value) {
        return StaticCallee.narrow((byte)value, (short)value, (char)value, value != 0);
    }
    public static int byteReturn(int value) { return StaticCallee.byteValue(value); }
    public static int shortReturn(int value) { return StaticCallee.shortValue(value); }
    public static int charReturn(int value) { return StaticCallee.charValue(value); }
    public static boolean booleanReturn(boolean value) { return StaticCallee.booleanValue(value); }
    public static int voidCall(int value) { StaticCallee.nothing(); return value; }
    public static void onlyVoid() { StaticCallee.nothing(); }
    public static int recursive(int n) { return n == 0 ? 0 : n + recursive(n - 1); }
    public static int mutual(int n) { return StaticCallee.mutual(n); }
    public static int accessProbe() { return StaticCallee.allowed(); }
    public static int nativeProbe() { return StaticCallee.nativeMethod(); }
    public static int synchronizedProbe() { return StaticCallee.synchronizedMethod(); }
    public static int initializationProbe() { return StaticInitialized.target(); }
    public static int divide(int a, int b) { return StaticCallee.divide(a, b); }
    public static int retryProbe() { StaticCallee.nativeVoid(); return 42; }
}

class StaticCallee {
    public static int combine(int a, int b) { return a * 10 + b; }
    public static int identity(int value) { return value; }
    public static long identity(long value) { return value; }
    public static float identity(float value) { return value; }
    public static double identity(double value) { return value; }
    public static long mixed(int a, long b, double c, float d, Object e) {
        return a + b + (long)c + (long)d + (e == null ? 7 : 9);
    }
    public static Object reference(Object value) { return value; }
    public static int[] array(int[] value) { return value; }
    public static int narrow(byte a, short b, char c, boolean d) {
        return a + b + c + (d ? 1 : 0);
    }
    public static byte byteValue(int value) { return (byte)value; }
    public static short shortValue(int value) { return (short)value; }
    public static char charValue(int value) { return (char)value; }
    public static boolean booleanValue(boolean value) { return value; }
    public static void nothing() { }
    public static int mutual(int n) { return n == 0 ? 0 : 1 + StaticCalls.mutual(n - 1); }
    public static int allowed() { return 11; }
    private static int blocked() { return 12; }
    public int regular() { return 13; }
    public static native int nativeMethod();
    public static native void nativeVoid();
    public static synchronized int synchronizedMethod() { return 14; }
    public static int divide(int a, int b) { return a / b; }
}

class StaticBase {
    public static int inherited(int value) { return value + 5; }
}
class StaticLeaf extends StaticBase { }
interface StaticOperation {
    static int apply(int value) { return value * 3; }
}
class StaticInitialized {
    static int value = initialize();
    private static int initialize() { return 42; }
    public static int target() { return 7; }
}
