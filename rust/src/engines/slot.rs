use crate::{gc_bindings::oop_hierarchy::NObjPtr, oops::jvalue::{JFloat, JInt}};

#[derive(Debug, Clone, Copy)]
pub enum Slot {
    Unused,
    Int(JInt),
    LongLow([u8; 4]),
    LongHigh([u8; 4]),
    Float(JFloat),
    DoubleLow([u8; 4]),
    DoubleHigh([u8; 4]),
    Ref(NObjPtr)
}
