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

class FailingParentInit {
    static int value = 1 / 0;
}

class ChildOfFailingParent extends FailingParentInit {
    static int read() {
        return 1;
    }
}

class OnceClassInit {
    static int count;

    static {
        count = count + 1;
    }

    static int read() {
        return count;
    }
}

class RecursiveClassInit {
    static int value = duringInit();

    static int duringInit() {
        return value + 1;
    }

    static int read() {
        return value;
    }
}

class AbortableClassInit {
    static int value = 6;

    static int read() {
        return value;
    }
}

class InterfaceInitTrace {
    static int value;

    static int record(int digit) {
        value = value * 10 + digit;
        return value;
    }
}

interface PlainParentInterface {
    int MARK = InterfaceInitTrace.record(9);
}

interface RootDefaultInterface {
    int MARK = InterfaceInitTrace.record(1);

    default int rootDefault() {
        return 1;
    }
}

interface ChildDefaultInterface extends PlainParentInterface, RootDefaultInterface {
    int MARK = InterfaceInitTrace.record(2);

    default int childDefault() {
        return 2;
    }
}

class DefaultInterfaceInitTarget implements ChildDefaultInterface {
    static int MARK = InterfaceInitTrace.record(3);

    static int initializationOrder() {
        return InterfaceInitTrace.value;
    }
}

class InterfaceSelfInitTrace {
    static int value;

    static int record(int digit) {
        value = value * 10 + digit;
        return value;
    }
}

interface ParentOfSelfInitializedInterface {
    int MARK = InterfaceSelfInitTrace.record(8);
}

interface SelfInitializedInterface extends ParentOfSelfInitializedInterface {
    int MARK = InterfaceSelfInitTrace.record(4);
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

    public static int readFailingChild() {
        return ChildOfFailingParent.read();
    }

    public static int readOnceClass() {
        return OnceClassInit.read();
    }

    public static int readRecursiveClass() {
        return RecursiveClassInit.read();
    }

    public static int defaultInterfaceInitializationOrder() {
        return DefaultInterfaceInitTarget.initializationOrder();
    }

    public static int initializeInterfaceWithoutParent() {
        int ignored = SelfInitializedInterface.MARK;
        return InterfaceSelfInitTrace.value;
    }
}
