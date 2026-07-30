public class ReferenceLoads {
    public static Object load0(Object value) {
        return value;
    }

    public static Object load1(int a, Object value) {
        return value;
    }

    public static Object load2(int a, int b, Object value) {
        return value;
    }

    public static Object load3(int a, int b, int c, Object value) {
        return value;
    }

    public static Object loadIndexed(int a, int b, int c, int d, Object value) {
        return value;
    }

    public static Object roundTrip(Object value) {
        Object local = value;
        return local;
    }
}
