//! Java 堆对象的压缩指针编解码 + markword 解码。
//!
//! 与 C 层 `core/memory/comp_space_defs.h` 和 `core/obj_model/markword.h` 对齐。
//!
//! 两个独立的压缩指针空间：
//!   - **对象引用**（栈槽、对象体内的 oop 字段）：以 `GCHEAP_BASE` 为基准，
//!     指向 Java 堆内的 `ObjDesc`。  对应 `encode_oop` / `decode_oop`。
//!   - **Klass 指针**（markword 里编码的）：以 `METASPACE_BASE` 为基准，
//!     指向 metaspace 内的 `Klass`。  统一走 `MSRef::encode` / `decode`。

use crate::class_loader::ms_api::MSRef;
use crate::gc_bindings::oop_handle::ObjDesc;
use crate::oops::klass::Klass;

/// Java 堆的虚拟内存基址（与 C 层 `GCHEAP_BASE` 一致）。
pub const GCHEAP_BASE: usize = 1usize << 44;

/// 压缩指针的对齐粒度（与 C 层 `COMP_PTR_SHIFT` 一致）。
const COMP_PTR_SHIFT: u32 = 3;

// ── markword 位域布局（与 `core/obj_model/markword.h` 一致）─────────────

/// markword 最低 2 位：lock value。
const LOCKVALUE_BITS: u32 = 2;
const LOCKVALUE_SHIFT: u32 = 0;

/// markword 高 32 位：klass compressed ptr（实际占用 bit 31..62）。
const KLASS_COMPPTR_BITS: u32 = 32;
const KLASS_COMPPTR_SHIFT: u32 = 31;

/// 将一个 Java 堆内的 `ObjDesc` 裸指针编码为 32 位 narrow ptr。
///
/// `ptr` 为 null 时返回 0。
///
/// # Panics
/// `ptr` 必须位于 `[GCHEAP_BASE, GCHEAP_BASE + 32GB)` 内且 8 字节对齐。
pub fn encode_oop(ptr: *const ObjDesc) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let addr = ptr as usize;
    assert!(
        addr >= GCHEAP_BASE,
        "encode_oop: ptr {:#x} below GCHEAP_BASE",
        addr
    );
    let off = addr - GCHEAP_BASE;
    let narrow = (off >> COMP_PTR_SHIFT) as u32;
    narrow
}

/// 将 32 位 narrow ptr 解码为 Java 堆内的 `ObjDesc` 裸指针。
///
/// `narrow == 0` 返回 null。
pub fn decode_oop(narrow: u32) -> *mut ObjDesc {
    if narrow == 0 {
        return std::ptr::null_mut();
    }
    let addr = GCHEAP_BASE + ((narrow as usize) << COMP_PTR_SHIFT);
    addr as *mut ObjDesc
}

/// 从 markword 读出 klass 的 compressed ptr（CompPtr，以 METASPACE_BASE 为基准）。
pub fn markword_read_klass_cp(raw: u64) -> u32 {
    let mask_in_place: u64 = ((1u64 << KLASS_COMPPTR_BITS) - 1) << KLASS_COMPPTR_SHIFT;
    ((raw & mask_in_place) >> KLASS_COMPPTR_SHIFT) as u32
}

/// 从 markword 读出 lock value。
#[allow(dead_code)]
pub fn markword_read_lock_value(raw: u64) -> u32 {
    let mask_in_place: u64 = ((1u64 << LOCKVALUE_BITS) - 1) << LOCKVALUE_SHIFT;
    ((raw & mask_in_place) >> LOCKVALUE_SHIFT) as u32
}

/// 从 markword 解码出 `MSRef<Klass>`（metaspace 内的 `Klass`）。
///
/// 不读 markword 的其它位（lock state 等）。
///
/// # Safety
/// markword 必须由 `gcheap_alloc` 写入（含合法的 narrow klass ptr）。
pub unsafe fn klass_from_markword(raw: u64) -> MSRef<Klass> {
    let cp = markword_read_klass_cp(raw);
    // SAFETY: 调用方保证 markword 来自合法分配的对象。
    unsafe { MSRef::decode(cp) }
}
