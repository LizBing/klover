#[repr(C)]
pub struct ObjLayout {
    pub super_layout: *const Self,
    
    // this size + super size
    pub byte_size: usize,
    
    pub ptr_count: usize,
}
