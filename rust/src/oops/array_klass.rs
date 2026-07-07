use crate::{gc_bindings::oop_handle::{NObjPtr, OOPHandle}, oops::{desc::{FieldDesc, FieldElemType}, symbol_table::SymbolHandle}};

/// 数组元素的固定布局：
///   markword(8) + length(4) + padding(4) + elements(...)
pub const ARRAY_HEADER_BYTES: usize = 16;
/// length 字段相对对象头的偏移。
pub const ARRAY_LENGTH_OFFSET: usize = 8;
/// 元素数据区相对对象头的偏移（length 后 padding 到 8）。
pub const ARRAY_DATA_OFFSET: usize = 16;

#[derive(Debug)]
pub struct ArrayKlass {
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    pub mirror: OOPHandle,
}

impl ArrayKlass {
    /// 数组元素的大小（字节）。
    ///
    /// `dimensions == 1` 时剥一层维度看 elem 的真实大小；
    /// `dimensions > 1` 时元素本身是子数组引用（narrow ptr，4 字节）。
    pub fn element_size(&self) -> usize {
        if self.desc.dimensions > 1 {
            // 元素是子数组引用。
            return size_of::<NObjPtr>();
        }
        // dimensions == 1：剥掉一维，看 elem 自身大小。
        match self.desc.elem {
            crate::oops::desc::FieldElemType::Boolean => 1,
            crate::oops::desc::FieldElemType::Byte => 1,
            crate::oops::desc::FieldElemType::Char => 2,
            crate::oops::desc::FieldElemType::Short => 2,
            crate::oops::desc::FieldElemType::Int => 4,
            crate::oops::desc::FieldElemType::Float => 4,
            crate::oops::desc::FieldElemType::Long => 8,
            crate::oops::desc::FieldElemType::Double => 8,
            // 引用类型元素（Class）占 4 字节（narrow ptr）。
            crate::oops::desc::FieldElemType::Class { .. } => {
                std::mem::size_of::<NObjPtr>()
            }
        }
    }

    /// 元素是否是引用类型（对象 / 子数组）。
    pub fn element_is_reference(&self) -> bool {
        self.desc.dimensions > 1
            || matches!(
                self.desc.elem,
                FieldElemType::Class { .. }
            )
    }
}
