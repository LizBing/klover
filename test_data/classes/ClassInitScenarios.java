class ClassInitBase {
    static int base = 40;
}

class ClassInitChild extends ClassInitBase {
    static int value = base + 2;
}

class PutStaticNeedsInit {
    static int value = 5;
}

class FailingClassInit {
    static int value = 1 / 0;

    static int read() {
        return value;
    }
}

public class ClassInitScenarios {
    public static int readChildField() {
        return ClassInitChild.value;
    }

    public static int writeThenRead(int value) {
        PutStaticNeedsInit.value = value;
        return PutStaticNeedsInit.value;
    }

    public static int readFailingClass() {
        return FailingClassInit.read();
    }
}
