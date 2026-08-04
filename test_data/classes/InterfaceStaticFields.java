class InterfaceFieldInitializer {
    static int value() {
        return 42;
    }
}

interface RootInterfaceField {
    int VALUE = InterfaceFieldInitializer.value();
}

interface MiddleInterfaceField extends RootInterfaceField {}

interface LeftInterfaceField extends RootInterfaceField {}

interface RightInterfaceField extends RootInterfaceField {}

public class InterfaceStaticFields
        implements MiddleInterfaceField, LeftInterfaceField, RightInterfaceField {
    public static int readThroughMiddle() {
        return MiddleInterfaceField.VALUE;
    }

    public static int readThroughClass() {
        return InterfaceStaticFields.VALUE;
    }
}
