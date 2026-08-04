class StaticFieldBase {
    static int inherited;
}

public class StaticFields extends StaticFieldBase {
    static boolean booleanValue;
    static byte byteValue;
    static char charValue;
    static short shortValue;
    static int intValue;
    static float floatValue;
    static long longValue;
    static double doubleValue;
    static Object refValue;

    public static int booleanRoundTrip(boolean value) {
        booleanValue = value;
        return booleanValue ? 1 : 0;
    }

    public static int byteRoundTrip(byte value) {
        byteValue = value;
        return byteValue;
    }

    public static int charRoundTrip(char value) {
        charValue = value;
        return charValue;
    }

    public static int shortRoundTrip(short value) {
        shortValue = value;
        return shortValue;
    }

    public static int intRoundTrip(int value) {
        intValue = value;
        return intValue;
    }

    public static float floatRoundTrip(float value) {
        floatValue = value;
        return floatValue;
    }

    public static long longRoundTrip(long value) {
        longValue = value;
        return longValue;
    }

    public static double doubleRoundTrip(double value) {
        doubleValue = value;
        return doubleValue;
    }

    public static Object defaultReference() {
        return refValue;
    }

    public static int inheritedRoundTrip(int value) {
        StaticFields.inherited = value;
        return StaticFields.inherited;
    }
}
