public class FieldLayout {
    static class Base {
        byte tag;
        long ticks;
        static Object shared;
        static byte flag;
        static long sequence;
        static short kind;
        static Object[] names;
        static int mask;
        static int[] indices;
        Object owner;
        float weight;
        char code;
    }

    static class Middle extends Base {
        int count;
    }

    static class Child extends Middle {
        boolean ready;
        long[][] matrix;
        double value;
        short delta;
        int[] owner;
    }

    static class Empty extends Child {
        static int[] cache;
    }
}
