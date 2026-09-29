// Compiled but skipped until invokestatic is supported. No VM changes required.
public class DeferredCalls {
    public static int twice(int x) { return x * 2; }
    public static int nested(int x) { return twice(twice(x)); }
    public static int crossClass(int x) { return StaticCallee.twice(x); }
}
