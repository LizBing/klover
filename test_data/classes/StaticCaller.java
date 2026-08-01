public class StaticCaller {
    public static int callInt(int left, int right) {
        return StaticCallee.twice(left + right);
    }

    public static long callLong(long left, long right) {
        return StaticCallee.addLong(left, right);
    }

    public static int callVoid(int value) {
        StaticCallee.consume(value);
        return value + 1;
    }

    public static int callNeedsClinit() {
        return StaticNeedsClinit.value();
    }
}
