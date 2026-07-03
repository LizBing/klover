//! 运行时常量池引用解析。
//!
//! 把 CP 里的 FieldRef / MethodRef / InterfaceMethodRef 解析成实际的
//! `&Field` / `&Method`，并写入 `CPRefEntry.resolved`（OnceLock 缓存）。
//!
//! 这里放在 `class_loader` 层而非 `oops` 层，是因为解析需要触发类加载
//! （走 caller 的 ClassLoader），而 `oops` 不能反向依赖 `class_loader`。

use crate::{
    class_loader::{bootstrap_cld::BootstrapCLD, ms_api::MSRef},
    oops::{
        cp_entry::{CPRefEntry, ClassCPEntry},
        field::Field,
        klass::Klass,
        method::Method,
        normal_klass::NormalKlass,
        resolve_error::{ResolveError, ResolveResult},
    },
};

/// 用发起解析的类的 ClassLoader 加载符号引用指向的类。
///
/// - `caller.cld == None`：caller 由 bootstrap 加载，走 `BootstrapCLD`。
/// - `caller.cld == Some(cld)`：走 `cld.load_class`（双亲委派，未来真正实现）。
///
/// 返回的 `MSRef<Klass>` 内部是裸指针，deref 即可拿到 `&Klass`。
/// 生命周期与 metaspace 内的 Klass 绑定（永久存活）。
pub fn load_class_by_caller(caller: &NormalKlass, name: &str) -> ResolveResult<MSRef<Klass>> {
    let klass_ref = match caller.cld {
        Some(cld) => unsafe { (*cld.as_ptr()).load_class(name) },
        None => BootstrapCLD::find_class(name),
    }
    .map_err(|_| ResolveError::ClassNotFound)?;
    Ok(klass_ref)
}

/// 解析 CP 的 Class 引用（`new` / `checkcast` / `instanceof` / `anewarray` 等使用）。
///
/// 用 caller 的 ClassLoader 加载目标类，并缓存到 `ClassCPEntry.resolved`。
/// 返回 `MSRef<Klass>`。
pub fn resolve_class_ref(
    caller: &NormalKlass,
    entry: &ClassCPEntry,
) -> ResolveResult<MSRef<Klass>> {
    if let Some(r) = entry.resolved.get() {
        return Ok(r.clone());
    }
    let klass_ref = load_class_by_caller(caller, entry.name.utf8())?;
    let _ = entry.resolved.set(klass_ref.clone());
    Ok(entry.resolved.get().unwrap().clone())
}

/// 解析方法引用（含 InterfaceMethodRef）。
///
/// `caller` 是发起解析的类（CP 引用所在的方法所属的类）。
/// 按 JVM 规范，用 caller 的 ClassLoader 加载目标类，然后沿继承链向上查找
/// 第一个匹配 name+desc 的方法。
///
/// 注意：本函数**不做虚方法派发**。`invokevirtual` 的派发在指令 handler 里
/// 基于运行时对象的 klass 单独处理；这里只负责找出"声明的方法"用于参数
/// slot 计算等静态信息。
///
/// 返回 `MSRef<Method>`：metaspace 内的 Method 永久存活，调用方 deref 即用。
pub fn resolve_method_ref(
    caller: &NormalKlass,
    entry: &CPRefEntry<Method>,
) -> ResolveResult<MSRef<Method>> {
    if let Some(r) = entry.resolved() {
        return Ok(r.clone());
    }

    let klass_ref = load_class_by_caller(caller, entry.class_name().utf8())?;
    let normal = klass_ref.as_normal().ok_or(ResolveError::MismatchCPType)?;

    // 沿继承链向上找第一个匹配的方法。
    let mut cur: Option<&NormalKlass> = Some(normal);
    while let Some(k) = cur {
        if let Some(m) = k.find_method(entry.name(), entry.desc()) {
            let m_ref: MSRef<Method> = m.into();
            let _ = entry.try_resolve(m_ref.clone());
            return Ok(entry.resolved().unwrap().clone());
        }
        cur = k.get_super();
    }

    Err(ResolveError::MethodNotFound)
}

/// 解析字段引用（FieldRef）。
///
/// `caller` 是发起解析的类（CP 引用所在的方法所属的类）。
/// 沿继承链查找 name + descriptor 匹配的字段。返回的字段同时覆盖 instance
/// 与 static，调用方按 `acc_flags.contains(ACC_STATIC)` 决定访问路径。
///
/// 返回 `MSRef<Field>`：metaspace 内的 Field 永久存活，调用方 deref 即用。
pub fn resolve_field_ref(
    caller: &NormalKlass,
    entry: &CPRefEntry<Field>,
) -> ResolveResult<MSRef<Field>> {
    if let Some(r) = entry.resolved() {
        return Ok(r.clone());
    }

    let klass_ref = load_class_by_caller(caller, entry.class_name().utf8())?;
    let normal = klass_ref.as_normal().ok_or(ResolveError::MismatchCPType)?;

    if let Some(f) = normal.find_field(entry.name(), entry.desc()) {
        let f_ref: MSRef<Field> = f.into();
        let _ = entry.try_resolve(f_ref.clone());
        return Ok(entry.resolved().unwrap().clone());
    }

    Err(ResolveError::FieldNotFound)
}
