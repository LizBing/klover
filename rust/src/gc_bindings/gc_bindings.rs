unsafe extern "C" {
    /// Caller must serialize initialization and avoid reinitializing a live heap.
    pub fn gc_init(xmx: usize) -> bool;
}
