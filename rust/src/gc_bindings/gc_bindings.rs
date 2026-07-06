//! Java 堆对象分配的 FFI 包装。
//!
//! 对应 C 层 `core/gc/gc_heap.c`。`gcheap_alloc` 接收一个 `Klass*`
//! （C 层只用它的地址做 compressed klass ptr 编码，不读任何字段），
//! 返回指向 `ObjDesc` 的裸指针——对象 markword 已写好，payload 已清零。

use crate::gc_bindings::obj_layout::ObjLayout;
use crate::oops::{klass::Klass, oop_handle::ObjDesc};
use std::ffi::c_void;

unsafe extern "C" {
    fn gc_init(xmx: usize);
    fn gcheap_alloc(klass: *const c_void, word_size: usize) -> *mut ObjDesc;
}

/// 在 Java 堆上分配一个对象。
///
/// `klass` 是 metaspace 内的 Klass 指针（C 层只用其地址做 narrow klass ptr 编码）。
/// `byte_size` 是整个对象的大小（含对象头）。
///
/// 返回的 `ObjDesc` 指针：
/// - `markword` 已写入（含 narrow klass ptr）
/// - `payload` 已清零（字段默认值）
///
/// # Panics
/// 当堆空间不足时返回的指针为 null；本函数把它转为 panic。
pub fn alloc_object(klass: *const crate::oops::klass::Klass, byte_size: usize) -> *mut ObjDesc {
    // gcheap_alloc 接收的是 word_size（HeapWord 单位，8 字节）。
    // 对象大小总是 8 字节对齐（markword 8B + 字段按 size bucket 排列 + 整体对齐）。
    debug_assert!(
        byte_size % 8 == 0,
        "object size must be 8-byte aligned: {}",
        byte_size
    );
    let word_size = byte_size / 8;
    let ptr = unsafe { gcheap_alloc(klass as *const c_void, word_size) };
    assert!(!ptr.is_null(), "gcheap_alloc: out of heap memory");
    ptr
}

/// 获取一个 `NormalKlass` 的 `ObjLayout` 指针，供 C 端 GC 遍历对象内引用。
///
/// `klass` 必须指向 `Klass::Normal`；其它变体返回 null。
///
/// # Safety
/// `klass` 必须是 metaspace 内合法分配的 `Klass` 指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn klover_obj_layout_of(klass: *const Klass) -> *const ObjLayout {
    if klass.is_null() {
        return std::ptr::null();
    }
    match unsafe { &*klass } {
        Klass::Normal(n) => n.get_obj_layout() as *const ObjLayout,
        _ => std::ptr::null(),
    }
}
