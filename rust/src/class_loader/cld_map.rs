use std::ptr::NonNull;

use parking_lot::Mutex;

use crate::class_loader::cld::ClassLoaderData;

/// 侵入式链表哨兵。
///
/// 对 `*mut ClassLoaderData` 的薄封装，手动实现 `Send` 以放入 static。
/// 安全性由外部的 `Mutex` 保证。
struct Head(*mut ClassLoaderData);

// SAFETY: Head 仅通过 HEAD 的 Mutex 访问，Mutex 提供了线程安全。
unsafe impl Send for Head {}

/// 全局 ClassLoaderData 侵入式链表头。
static HEAD: Mutex<Head> = Mutex::new(Head(std::ptr::null_mut()));

// ── 公开 API ────────────────────────────────────────────────────────────

/// 将 CLD 插入全局链表头部。
///
/// 调用者负责确保同一个 CLD 不会被重复注册，且在注册后
/// `ClassLoaderData` 不会被移动（通常它存在于堆/metaspace 中）。
pub fn register(cld: NonNull<ClassLoaderData>) {
    let mut head = HEAD.lock();
    unsafe {
        (*cld.as_ptr()).next = head.0;
    }
    head.0 = cld.as_ptr();
}

/// 从全局链表中移除指定 CLD。
///
/// 如果 CLD 不在链表中，此调用无效果。
pub fn unregister(cld: NonNull<ClassLoaderData>) {
    let mut head = HEAD.lock();
    let target = cld.as_ptr();

    // 链表头就是目标
    unsafe {
        if head.0 == target {
            head.0 = (*target).next;
            return;
        }
    }

    // 遍历链表
    let mut cur = head.0;
    while !cur.is_null() {
        unsafe {
            let next = (*cur).next;
            if next == target {
                (*cur).next = (*target).next;
                return;
            }
            cur = next;
        }
    }
}

/// 遍历所有存活的 CLD。
///
/// 回调在锁内执行，应尽量简短，避免死锁。
pub fn for_each(mut f: impl FnMut(NonNull<ClassLoaderData>)) {
    let head = HEAD.lock();
    let mut cur = head.0;
    while !cur.is_null() {
        let cld = unsafe { NonNull::new_unchecked(cur) };
        f(cld);
        cur = unsafe { (*cur).next };
    }
}

/// 返回当前注册的 CLD 数量（调试用）。
pub fn count() -> usize {
    let head = HEAD.lock();
    let mut n = 0;
    let mut cur = head.0;
    while !cur.is_null() {
        n += 1;
        cur = unsafe { (*cur).next };
    }
    n
}
