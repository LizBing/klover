class StaticResolveTarget {
    public static int add(int a, int b) {
        return a + b;
    }
}

public class StaticResolveCaller {
    public static int call(int a, int b) {
        return StaticResolveTarget.add(a, b);
    }
}
