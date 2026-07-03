public class IntAdder implements Adder {

    public int add(int a, int b) {
        return a + b;
    }

    public static int interfaceTest() {
        Adder a = new IntAdder();
        return a.add(3, 4);
    }

    public static int interfaceArgTest(Adder a, int x, int y) {
        return a.add(x, y);
    }

    // 嵌套调用：interfaceTest 创建 IntAdder 并作为参数传入 interfaceArgTest。
    public static int nestedInterfaceTest() {
        Adder a = new IntAdder();
        return interfaceArgTest(a, 10, 20);
    }
}
