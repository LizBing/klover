//! 普通对象的内存布局描述。
//!
//! 分配在 metaspace（`NormalKlass` 的一个字段），`#[repr(C)]` 让 C 端 GC 可读。
//! C 端通过 FFI helper `klover_obj_layout_of` 获取，不走 enum 直接 cast。
//!
//! 布局含义见 `normal_klass.rs` 的 `Fields::build` 注释和
//! `core/obj_model/obj_layout.h` 的 GC 扫描算法文档。

#[repr(C)]
#[derive(Debug)]
pub struct ObjLayout {
    /// 父类的 layout。`java.lang.Object` 此字段为 null。
    pub super_layout: *const ObjLayout,

    /// 累计大小：markword + 所有父类部分 + 本类部分 + padding。
    /// 单位：字节，始终 8 字节对齐。
    /// 用于 `gcheap_alloc` 计算分配大小。
    pub byte_size: usize,

    /// 本类部分（非累计）的引用字段数量。
    /// 每个 oop 占 4 字节（NObjPtr），连续排列在本类部分开头。
    /// GC 遍历：本层 oop 区起点 = `super_layout.byte_size`（首层是 8，跳过 markword）。
    pub ptr_count: usize,
}

impl ObjLayout {
    /// 占位构造：在 `set_super` 之前先用零值占位。
    pub(crate) fn placeholder() -> Self {
        Self::default()
    }
}

impl Default for ObjLayout {
    fn default() -> Self {
        Self {
            super_layout: std::ptr::null(),
            byte_size: 0,
            ptr_count: 0,
        }
    }
}
