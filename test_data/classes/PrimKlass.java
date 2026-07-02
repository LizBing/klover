public class PrimKlass {
    public static void test01() {
        Class<?> int_class = int.class;
        System.out.println(int_class.getName());
    }

    public static void test02() {
        Class<?> int_array_class = int[].class;
        System.out.println(int_array_class.getName());
    }

    public static void main(String[] args) {
        test01();
    }
}
