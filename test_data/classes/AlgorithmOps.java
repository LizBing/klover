// Single-method algorithms: no invocation, arrays, fields, or constant-pool loads.
public class AlgorithmOps {
    public static long fibonacci(int n) {
        long previous = 0L;
        long current = 1L;
        for (int i = 0; i < n; i++) {
            long next = previous + current;
            previous = current;
            current = next;
        }
        return previous;
    }

    // Inputs are nonnegative; gcd(0, 0) is defined as zero for this fixture.
    public static int gcd(int a, int b) {
        while (b != 0) {
            int remainder = a % b;
            a = b;
            b = remainder;
        }
        return a;
    }

    public static boolean isPrime(int n) {
        if (n < 2) return false;
        for (int divisor = 2; divisor <= n / divisor; divisor++) {
            if (n % divisor == 0) return false;
        }
        return true;
    }

    public static int bitCount(int value) {
        int count = 0;
        while (value != 0) {
            count += value & 1;
            value >>>= 1;
        }
        return count;
    }

    public static int nestedSum(int rows, int columns) {
        int sum = 0;
        for (int row = 0; row < rows; row++) {
            for (int column = 0; column < columns; column++) {
                sum += row + column;
            }
        }
        return sum;
    }

    public static double polynomial(double x, double a, double b, double c) {
        return (a * x + b) * x + c;
    }

    public static void spin() {
        while (true) { }
    }
}
