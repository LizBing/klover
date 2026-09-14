use crate::gc_bindings::oop_hierarchy::NObjPtr;

pub type JBoolean = u8;
pub type JByte = i8;
pub type JShort = i16;
pub type JChar = u16;
pub type JInt = i32;
pub type JLong = i64;
pub type JFloat = f32;
pub type JDouble = f64;

#[derive(Debug, Clone, Copy)]
pub enum JValue {
    Boolean(u8),
    Byte(JByte),
    Short(JShort),
    Char(JChar),
    Int(JInt),
    Long(JLong),
    Float(JFloat),
    Double(JDouble),
    Ref(NObjPtr)
}
