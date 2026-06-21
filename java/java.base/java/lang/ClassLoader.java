package java.lang;

public abstract class ClassLoader {
    private final ClassLoader parent;

    private final transient long native_cld_ptr;
    
    protected ClassLoader(ClassLoader parent) {
        this.parent = parent;
        native_cld_ptr = newNativeCLD();
    }

    protected Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
        Class<?> c = findLoadedClass(name);
        if (c == null) {
            try {
                if (parent != null) {
                    c = parent.findClass(name);
                } else {
                    c = findBootstrapClass(name);
                }
            } catch (ClassNotFoundException e) {
                // Not found in super classes. Ignore.
            }
            
            if (c == null) {
                c = findClass(name);
            }
        }

        return c;
    }

    protected Class<?> findClass(String name) throws ClassNotFoundException {
        throw new ClassNotFoundException(name);
    }

    protected Class<?> defineClass(String name, byte[] b, int off, int len) {
        return defineClass1(name, b, off, len);
    }

    protected native long newNativeCLD();
    protected native Class<?> defineClass1(String name, byte[] b, int off, int len);
    protected native Class<?> findLoadedClass(String name);
    protected native Class<?> findBootstrapClass(String name);
}
