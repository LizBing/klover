public class ControlFlow {
    // while 循环：1+2+...+n
    public static int sum(int n) {
        int total = 0;
        int i = 1;
        while (i <= n) {
            total += i;
            i++;
        }
        return total;
    }

    // if/else 分支
    public static int max(int a, int b) {
        if (a >= b) {
            return a;
        } else {
            return b;
        }
    }

    // 前缀递归式循环：for 计数
    public static int countTo(int n) {
        int c = 0;
        for (int i = 0; i < n; i++) {
            c++;
        }
        return c;
    }

    // do-while（至少执行一次）
    public static int factorial(int n) {
        int r = 1;
        int i = 1;
        do {
            r *= i;
            i++;
        } while (i <= n);
        return r;
    }

    // goto 无限循环 + break（编译器对某些结构会生成 goto）
    public static int firstEven(int from) {
        int i = from;
        while (true) {
            if (i % 2 == 0) {
                return i;
            }
            i++;
        }
    }
}
