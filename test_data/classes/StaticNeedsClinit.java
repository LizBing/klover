public class StaticNeedsClinit {
    private static int initialized = 1;

    public static int value() {
        return initialized;
    }
}
