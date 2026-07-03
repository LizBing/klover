public class ObjTest {

    int x;
    int y;

    public ObjTest(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public int sum() {
        return x + y;
    }

    public static int setAndGetX() {
        ObjTest o = new ObjTest(7, 9);
        return o.x;
    }

    public static int createAndSum(int a, int b) {
        ObjTest o = new ObjTest(a, b);
        return o.sum();
    }

    public static int getStaticTest() {
        return 42;
    }

    // newarray + iastore + iaload + arraylength
    public static int newArrayTest() {
        int[] arr = new int[5];
        arr[0] = 10;
        arr[1] = 20;
        return arr[0] + arr[1] + arr.length;
    }

    // instanceof + checkcast
    public static int instanceofTest() {
        ObjTest o = new ObjTest(1, 2);
        if (o instanceof ObjTest) return 1;
        return 0;
    }

    public static int instanceofNullTest() {
        ObjTest o = null;
        if (o instanceof ObjTest) return 1;
        return 0;
    }

    public static int checkcastTest() {
        Object o = new ObjTest(3, 4);
        ObjTest ot = (ObjTest) o;
        return ot.sum();
    }

    // anewarray + aastore + aaload
    public static int refArrayTest() {
        ObjTest[] arr = new ObjTest[2];
        arr[0] = new ObjTest(3, 4);
        arr[1] = new ObjTest(5, 6);
        return arr[0].sum() + arr[1].sum();
    }

    // 多种元素类型的读写
    public static int mixedArrayTest() {
        byte[] b = new byte[3];
        b[0] = -1;
        b[1] = 100;
        char[] c = new char[2];
        c[0] = 65;
        c[1] = 66;
        long[] l = new long[2];
        l[0] = 10000000000L;
        l[1] = 1L;
        return b[0] + b[1] + c[0] + c[1] + (int) (l[0] / l[1]);
    }

    // 仅验证 new + dup + pop 不崩。
    public static int allocOnly() {
        ObjTest o = new ObjTest(0, 0);
        return 0;
    }
}
