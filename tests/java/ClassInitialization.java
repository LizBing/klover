public class ClassInitialization {
    static int events;
    static int record(int digit) { events = events * 10 + digit; return digit; }

    static class Base { static int base = record(1); }
    static class Child extends Base { static int child = record(2); }
    public static int order() { return Child.child * 100 + events; }

    static class Self {
        static int seen = read();
        static int value = 9;
        static int read() { return value + 3; }
    }
    public static int recursive() { return Self.seen * 100 + Self.value; }
    static class A { static int x = B.y + 1; }
    static class B { static int y = A.x + 1; }
    public static int cycle() { return A.x * 10 + B.y; }

    static class Store { static int initialized = record(3); static long value; }
    public static long write(long value) { Store.value = value; return Store.value; }
    static class Call { static int seed = 7; static long add(long value) { return value + seed; } }
    public static long call(long value) { return Call.add(value); }

    interface Parent { int VALUE = record(4); default int first() { return 1; } }
    interface Sub extends Parent { int SECOND = record(5); default int second() { return 2; } }
    static class Defaults implements Sub { static int value = record(6); }
    public static int defaults() { events = 0; return Defaults.value * 1000 + events; }
    interface PlainParent { int VALUE = record(7); }
    interface Plain extends PlainParent { int SECOND = record(8); }
    public static int interfaceOnly() { events = 0; return Plain.SECOND * 10 + events; }

    static class Direct {
        static int runs;
        static int value = initialize();
        static int initialize() { runs++; return 17; }
        static int read() { return runs * 100 + value; }
    }

    static class Constants {
        static final byte B = -7;
        static final char C = 65535;
        static final short S = -300;
        static final int I = 42;
        static final long J = -123456789123L;
        static final float F = -0.0f;
        static final double D = 2.5;
        static int entry() { return I; }
    }
    static class Text { static final String VALUE = "text"; static int entry() { return 1; } }

    static class Failure { static int zero; static int value = 10 / zero; }
    public static int failure() { return Failure.value; }
    static class FailedChild extends Failure { static int child; }
    public static int dependencyFailure() { return FailedChild.child; }

    static class Slow {
        static int count;
        static int value = compute();
        static int compute() { count++; int sum = 0; for (int i = 0; i < 100; i++) sum += i; return sum; }
    }
    public static int slow() { return Slow.value; }
}
