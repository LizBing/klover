//! 运行时常量池引用解析。
//!
//! 把 CP 里的 FieldRef / MethodRef / InterfaceMethodRef 解析成实际的
//! `&Field` / `&Method`，并写入 `CPRefEntry.resolved`（OnceLock 缓存）。
//!
//! 这里放在 `class_loader` 层而非 `oops` 层，是因为解析需要触发类加载
//! （走 caller 的 ClassLoader），而 `oops` 不能反向依赖 `class_loader`。
