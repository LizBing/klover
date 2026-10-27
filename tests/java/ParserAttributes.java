import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import java.util.List;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.TYPE, ElementType.FIELD, ElementType.METHOD, ElementType.PARAMETER})
@interface ParserVisible {
    int number() default 7;
    String text() default "default";
    Class<?> type() default void.class;
    RetentionPolicy policy() default RetentionPolicy.RUNTIME;
    int[] values() default {1, 2};
}

@Retention(RetentionPolicy.CLASS)
@Target({ElementType.TYPE, ElementType.FIELD, ElementType.METHOD, ElementType.PARAMETER})
@interface ParserInvisible {}

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER})
@interface ParserType {}

@ParserVisible(number = 42, text = "class")
@ParserInvisible
@Deprecated
public class ParserAttributes<@ParserType T extends @ParserType Object> {
    public static final int NUMBER = 42;
    public static final String TEXT = "literal";

    @ParserVisible @ParserInvisible
    public List<@ParserType T> field;

    @ParserVisible @ParserInvisible
    public @ParserType String run(@ParserVisible @ParserInvisible final int input)
            throws @ParserType Exception {
        List<T> genericLocal = field;
        @ParserType Object local = new @ParserType Object();
        try {
            if (input < 0) throw new Exception();
            return (@ParserType String) local;
        } catch (@ParserType Exception error) {
            return "caught";
        }
    }

    public static Runnable lambda() {
        return () -> {};
    }

    public static Object localClass() {
        class Local {}
        return new Local();
    }

    public static class Nested {}
}
