// Switch fixtures avoid ldc, method calls, and object allocation.
public class SwitchOps {
    public static int dense(int key) {
        switch (key) {
            case -1: return 11;
            case 0: return 22;
            case 1: return 33;
            case 2: return 44;
            default: return 99;
        }
    }

    public static int denseIndexed(int a, int b, int c, int d, int key) {
        switch (key) {
            case -1: return 11;
            case 0: return 22;
            case 1: return 33;
            case 2: return 44;
            default: return 99;
        }
    }

    public static int denseAddOne(int key) {
        switch (key + 1) {
            case -1: return 11;
            case 0: return 22;
            case 1: return 33;
            case 2: return 44;
            default: return 99;
        }
    }

    public static int denseAddSix(int key) {
        switch (key + 6) {
            case -1: return 11;
            case 0: return 22;
            case 1: return 33;
            case 2: return 44;
            default: return 99;
        }
    }

    public static int sparse(int key) {
        switch (key) {
            case -100: return 11;
            case 7: return 22;
            case 1000: return 33;
            default: return 99;
        }
    }

    public static int sparseIndexed(int a, int b, int c, int d, int key) {
        switch (key) {
            case -100: return 11;
            case 7: return 22;
            case 1000: return 33;
            default: return 99;
        }
    }

    public static int sparseAddOne(int key) {
        switch (key + 1) {
            case -100: return 11;
            case 7: return 22;
            case 1000: return 33;
            default: return 99;
        }
    }

    public static int sparseAddSix(int key) {
        switch (key + 6) {
            case -100: return 11;
            case 7: return 22;
            case 1000: return 33;
            default: return 99;
        }
    }

    public static int denseHole(int key) {
        switch (key) {
            case -2: return 10;
            case -1: return 20;
            case 1: return 30;
            case 2: return 40;
            default: return 99;
        }
    }

    public static int denseFallThrough(int key) {
        int result = 0;
        switch (key) {
            case 0: result += 1;
            case 1: result += 2;
            case 2:
            case 3: result += 4; break;
            default: result = 99;
        }
        return result;
    }

    public static int sparseFallThrough(int key) {
        int result = 0;
        switch (key) {
            case -100: result += 1;
            case 7: result += 2;
            case 1000:
            case 10000: result += 4; break;
            default: result = 99;
        }
        return result;
    }

    public static int denseMin(int key) {
        switch (key) {
            case -2147483648: return 1;
            case -2147483647: return 2;
            case -2147483646: return 3;
            default: return 99;
        }
    }

    public static int denseMax(int key) {
        switch (key) {
            case 2147483645: return 1;
            case 2147483646: return 2;
            case 2147483647: return 3;
            default: return 99;
        }
    }

    public static int sparseExtremes(int key) {
        switch (key) {
            case -2147483648: return 1;
            case 0: return 2;
            case 2147483647: return 3;
            default: return 99;
        }
    }

    public static int defaultOnly(int key) {
        switch (key) {
            default: return 99;
        }
    }

    public static int denseLoop(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 4) {
                case 0: total += 1; break;
                case 1: total += 2; break;
                case 2: total += 3; break;
                default: total += 4;
            }
        }
        return total;
    }

    public static int sparseLoop(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 4) {
                case -100: total += 1; break;
                case 1: total += 2; break;
                case 1000: total += 3; break;
                default: total += 4;
            }
        }
        return total;
    }
}
