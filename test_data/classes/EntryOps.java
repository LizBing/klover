// Parameters and local variables prevent javac from folding away arithmetic.
public class EntryOps {
    public static int test() { int a = 1; int b = 2; return a + b; }
    public static byte byteIdentity(byte x) { return x; }
    public static char charIdentity(char x) { return x; }
    public static short shortIdentity(short x) { return x; }
    public static boolean booleanIdentity(boolean x) { return x; }
    public static double mixed(int a, long b, float c, double d) {
        double local = a + b;
        return local + c + d;
    }
}
