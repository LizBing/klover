import java.lang.invoke.MethodType;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

/** Runs only on the reference JVM; none of this wrapper executes in Klover. */
public final class HotSpotRunner {
    private static Object argument(char type, String value) {
        switch (type) {
            case 'Z':
                if (!value.equals("0") && !value.equals("1"))
                    throw new IllegalArgumentException("boolean must be 0 or 1");
                return value.equals("1");
            case 'B': return Byte.valueOf(value);
            case 'C':
                int c = Integer.parseInt(value);
                if (c < 0 || c > 65535) throw new IllegalArgumentException("char out of range");
                return (char) c;
            case 'S': return Short.valueOf(value);
            case 'I': return Integer.valueOf(value);
            case 'J': return Long.valueOf(value);
            case 'F': return Float.intBitsToFloat(Integer.parseUnsignedInt(value, 16));
            case 'D': return Double.longBitsToDouble(Long.parseUnsignedLong(value, 16));
            default: throw new IllegalArgumentException("unsupported parameter type");
        }
    }

    private static String result(char type, Object value) {
        switch (type) {
            case 'V': return "void";
            case 'Z': return (Boolean) value ? "1" : "0";
            case 'C': return Integer.toString((Character) value);
            case 'F':
                float f = (Float) value;
                return Float.isNaN(f) ? "nan" : String.format("%08x", Float.floatToRawIntBits(f));
            case 'D':
                double d = (Double) value;
                return Double.isNaN(d) ? "nan" : String.format("%016x", Double.doubleToRawLongBits(d));
            default: return Long.toString(((Number) value).longValue());
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("--vm-info")) {
            System.out.println(System.getProperty("java.vm.name"));
            System.out.println(System.getProperty("java.vm.version"));
            System.out.println(System.getProperty("java.version"));
            return;
        }
        if (args.length < 3 || !args[2].matches("\\([ZBCSIJFD]*\\)[ZBCSIJFDV]"))
            throw new IllegalArgumentException("usage: CLASS METHOD PRIMITIVE_DESCRIPTOR [ARGS...]");
        String desc = args[2];
        int count = desc.indexOf(')') - 1;
        if (args.length != count + 3) throw new IllegalArgumentException("argument count mismatch");
        MethodType signature = MethodType.fromMethodDescriptorString(desc, HotSpotRunner.class.getClassLoader());
        Class<?> owner = Class.forName(args[0].replace('/', '.'));
        Method method = owner.getDeclaredMethod(args[1], signature.parameterArray());
        if (!Modifier.isStatic(method.getModifiers()) || !Modifier.isPublic(method.getModifiers())
                || method.getReturnType() != signature.returnType())
            throw new IllegalArgumentException("entry must be public static with matching return type");
        Object[] values = new Object[count];
        for (int i = 0; i < count; i++) values[i] = argument(desc.charAt(i + 1), args[i + 3]);
        Object value = method.invoke(null, values);
        char type = desc.charAt(desc.length() - 1);
        System.out.println("KLOVER_DIFF_V1\t" + type + "\t" + result(type, value));
    }
}
