#[derive(Debug)]
pub enum FieldType {}

pub enum ReturnDesc {
    Void,
    Type(FieldType)
}
