//! Byte offsets for one class's instance or static field segment.

use crate::gc_bindings::oop_hierarchy::NObjPtr;

/// The native ObjDesc header consists of one 64-bit mark word.
pub(crate) const OBJECT_HEADER_SIZE: usize = size_of::<u64>();
pub(crate) const OBJECT_ALIGNMENT: usize = 8;

#[repr(C)]
#[derive(Debug)]
pub struct ObjLayout {
    /// Start of the class's segment relative to the object or static block.
    pub begin_offs: usize,
    /// Aligned end offset, also the complete instance or static block size.
    pub byte_size: usize,
    /// Number of consecutive compressed reference slots declared by this class.
    pub ptrs_count: usize,
}

impl ObjLayout {
    pub fn reference_offsets(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.ptrs_count).map(|index| self.begin_offs + index * size_of::<NObjPtr>())
    }
}
