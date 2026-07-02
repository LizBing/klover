use std::{ops::Deref, ptr::NonNull};

use dashmap::{DashMap, mapref::entry::Entry};

use crate::{
    class_loader::{
        class_path::ClassPath,
        cld_map,
        load_error::{LoadError, LoadResult},
        ms_box::{MSAllocator, MSBox},
    },
    class_parser::{class_file::ClassFile, cp_info::ConstantPoolInfo},
    oops::{
        klass::Klass, normal_klass::NormalKlass, oop_handle::{CLD_MIRROR_STORAGE_ID, OOPHandle}, resolve_error::ResolveError, symbol_table::{SymbolHandle, SymbolTable}
    },
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
    // ── 构造 ───────────────────────────────────────────────────────────

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
    /// 从 `&MSBox<Klass>` 通过 Deref 获取 `NonNull<Klass>`，不转移所有权。
    fn klass_ptr(mb: &MSBox<Klass>) -> NonNull<Klass> {
        let r: &Klass = mb;
        unsafe { NonNull::new_unchecked(r as *const Klass as *mut Klass) }
    }

    pub fn define_class(&self, bytes: &[u8]) -> LoadResult<NonNull<Klass>> {
        let cf = match ClassFile::from(bytes) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Parse(e)),
        };

        let name_utf8 = match &cf.constant_pool[cf.this_class as usize] {
            ConstantPoolInfo::ClassInfo { name_index } => 
                match &cf.constant_pool[*name_index as usize] {
                    ConstantPoolInfo::Utf8Info { utf8 } => utf8.clone(),
                    _ => return Err(LoadError::Resolve(ResolveError::MismatchCPType))
                },

            _ => return Err(LoadError::Resolve(ResolveError::MismatchCPType))
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

        let klass = match NormalKlass::from(cf, &self.ms_allocator) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Resolve(e)),
        };
        let self_ptr = unsafe { NonNull::new_unchecked(self as *const _ as _) };
        klass.cld.set(Some(self_ptr)).unwrap();

        let boxed = MSBox::new(&self.ms_allocator, Klass::Normal(klass));
        let r = vacant.insert(boxed);

        Ok(Self::klass_ptr(r.deref()))
    }

    pub fn find_loaded_class(&self, name: &str) -> Option<NonNull<Klass>> {
        let sym = SymbolTable::intern(name);
        
        match self.klasses.get(&sym) {
            Some(x) => Some(Self::klass_ptr(x.deref())),
            None => None
        }
    }
}
