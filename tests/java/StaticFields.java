public class StaticFields {
    static boolean z;
    static byte b;
    static char c;
    static short s;
    static int i;
    static volatile long j;
    static float f;
    static double d;
    static Object r;
    static int[] a;
    private static int hidden;
    public int instance;
    static class Hidden { public static int value; }
    public static class Constants { public static final int CONSTANT = 42; }

    public static boolean bool(boolean v) { z = v; return z; }
    public static int narrow(int v) { b = (byte)v; c = (char)v; s = (short)v; return b + c + s; }
    public static int integer(int v) { i = v; return i; }
    public static long wide(long v) { j = v; return j; }
    public static float floating(float v) { f = v; return f; }
    public static double precise(double v) { d = v; return d; }
    public static Object reference(Object v) { r = v; return r; }
    public static int[] array(int[] v) { a = v; return a; }
    public static int inherited(int v) { Child.shared = v; return Base.shared; }
    public static int initialized() { return Initialized.value; }

    public static class Base { public static int shared; }
    public static class Child extends Base { static int unrelated = 7; }
    public static class Initialized { static int value = 7; }
    public interface Root { int TOKEN = 12; int shared = 13; }
    public interface Sub extends Root {}
    public static class Implements extends Base implements Sub {}
}
