use std::{ops::Deref, ptr::NonNull};

use dashmap::{DashMap, mapref::entry::Entry};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld_map,
        load_error::{LoadError, LoadResult},
        ms_api::{MSAllocator, MSBox, MSRef},
    }, class_parser::{class_file::ClassFile, cp_info::ConstantPoolInfo}, gc_bindings::oop_handle::{CLD_MIRROR_STORAGE_ID, OOPHandle}, oops::{
        klass::Klass, normal_klass::{NormalKlass, UnlinkedNormalKlass}, resolve_error::ResolveError, symbol_table::{SymbolHandle, SymbolTable},
    }
};

// ── ClassLoaderData ─────────────────────────────────────────────────────

pub struct ClassLoaderData {
    pub(super) next: *mut ClassLoaderData,

    pub mirror: OOPHandle,
    pub debug_name: Option<String>,

    pub ms_allocator: MSAllocator,
    klasses: DashMap<SymbolHandle, MSBox<Klass>>,
}

unsafe impl Send for ClassLoaderData {}
unsafe impl Sync for ClassLoaderData {}

impl Drop for ClassLoaderData {
    fn drop(&mut self) {
        cld_map::unregister(NonNull::from(self));
    }
}

impl ClassLoaderData {
    // ── 构造 ──────────────────────────────────────────────────────────
    pub fn new(debug_name: Option<String>) -> NonNull<Self> {
        let cld = Box::new(Self {
            next: std::ptr::null_mut(),
            mirror: OOPHandle::new(CLD_MIRROR_STORAGE_ID),
            debug_name,
            ms_allocator: MSAllocator::new(),
            klasses: DashMap::new(),
        });

        let ptr: NonNull<Self> = Box::leak(cld).into();
        cld_map::register(ptr);

        ptr
    }
}

impl ClassLoaderData {
    pub fn define_class(&self, bytes: &[u8]) -> LoadResult<MSRef<Klass>> {
        let cf = match ClassFile::from(bytes) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Parse(e)),
        };

        let name_utf8 = match &cf.constant_pool[cf.this_class as usize] {
            ConstantPoolInfo::ClassInfo { name_index } => {
                match &cf.constant_pool[*name_index as usize] {
                    ConstantPoolInfo::Utf8Info { utf8 } => utf8.clone(),
                    _ => return Err(LoadError::Resolve(ResolveError::MismatchCPType)),
                }
            }

            _ => return Err(LoadError::Resolve(ResolveError::MismatchCPType)),
        };
        // field desc
        let name = SymbolTable::intern(name_utf8.as_str());

        let entry = self.klasses.entry(name);
        let vacant = match entry {
            Entry::Occupied(_) => {
                return Err(LoadError::Duplicated {
                    cld_name: self.debug_name.clone(),
                    class_name: name_utf8,
                });
            }

            Entry::Vacant(v) => v,
        };

        let unlinked = match UnlinkedNormalKlass::build(cf, Some(&self)) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Resolve(e)),
        };

        let normal = NormalKlass::link(unlinked, Some(self))
            .map_err(|e| LoadError::Resolve(e))?;

        let res = (&normal).into();
        vacant.insert(normal);

        Ok(res)
    }

    pub fn find_loaded_class(&self, name: &str) -> Option<MSRef<Klass>> {
        let sym = SymbolTable::intern(name);

        match self.klasses.get(&sym) {
            Some(x) => Some(x.deref().into()),
            None => None,
        }
    }
}

impl ClassLoaderData {
    /// 加载指定名称的类。
    ///
    /// 按双亲委派模型：先委派给父加载器，最终落到 bootstrap。
    /// 当前实现只是把所有请求委派给 `BootstrapCLD`（尚未支持用户自定义 ClassLoader
    /// 的 `loadClass` 覆盖）。未来接入 native `ClassLoader.loadClass` 时重写。
    pub fn load_class(&self, name: &str) -> LoadResult<MSRef<Klass>> {
        // TODO: 真正的双亲委派（调用 self.mirror 对应的 java/lang/ClassLoader.loadClass）。
        BootstrapCLD::find_class(name)
    }
}
