public class InterfaceLoading {
    public interface Root {}
    public interface First extends Root {}
    public interface Second {}
    public abstract static class Base implements First, Second {}
    public abstract static class Leaf extends Base {}
}
