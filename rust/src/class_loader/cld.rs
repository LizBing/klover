use std::ptr::NonNull;

#[repr(C)]
struct MSChunk;

unsafe extern "C" {
    fn ms_alloc_small_chunk() -> *mut MSChunk;
    fn ms_alloc_sized_chunk(byte_size: usize) -> *mut MSChunk;
    fn ms_free_chunk(n: *mut MSChunk);
}

struct ClassLoaderData {
    chunks: Vec<NonNull<MSChunk>>
}

impl Drop for ClassLoaderData {
    fn drop(&mut self) {
        for n in &self.chunks {
            unsafe { ms_free_chunk(n.as_ptr()); }
        }
    }
}
