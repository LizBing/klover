public class MethodResolution {
    public static class Base {
        public static int inherited() { return 1; }
        public static int overloaded(int value) { return value; }
        public static long overloaded(long value) { return value; }
        public static int hidden() { return 2; }
    }

    public static class Child extends Base {
        public static int hidden() { return 3; }
    }

    public static class Leaf extends Child {}

    public interface Root {
        static int declared() { return 4; }
        int abstractMethod();
        default int defaultMethod() { return 5; }
    }

    public interface Subinterface extends Root {}

    public abstract static class Implementor implements Root {}

    public abstract static class Subimplementor extends Implementor {}
}
