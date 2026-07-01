use std::{ptr::NonNull, thread};

use dashmap::{DashMap, mapref::entry::Entry};

use crate::{
    class_loader::{
        class_path::ClassPath,
        cld_map,
        load_error::{LoadError, LoadResult},
        ms_box::{MSAllocator, MSBox},
    },
    class_parser::class_file::ClassFile,
    oops::{
        cp_entry::ClassCPEntry,
        klass::Klass,
        normal_klass::NormalKlass,
        oop_handle::{CLD_MIRROR_STORAGE_ID, OOPHandle},
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

// ── ClassEntry ──────────────────────────────────────────────────────────

enum ClassEntry {
    Loading,
    Done {
        klass: MSBox<Klass>,
        super_entry: Option<NonNull<ClassCPEntry>>,
    },
}

// ── ClassLoaderData ─────────────────────────────────────────────────────

pub struct ClassLoaderData {
    pub(super) next: *mut ClassLoaderData,
    mirror: OOPHandle,
    pub debug_name: Option<String>,

    class_path: Box<dyn ClassPath>,

    pub ms_allocator: MSAllocator,
    klasses: DashMap<SymbolHandle, ClassEntry>,

    waiters: DashMap<SymbolHandle, Vec<thread::Thread>>,
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

    pub fn new(debug_name: Option<String>, class_path: Box<dyn ClassPath>) -> NonNull<Self> {
        let cld = Box::new(Self {
            next: std::ptr::null_mut(),
            mirror: OOPHandle::new(CLD_MIRROR_STORAGE_ID),
            debug_name,
            class_path,
            ms_allocator: MSAllocator::new(),
            klasses: DashMap::new(),
            waiters: DashMap::new(),
        });
        let ptr: NonNull<Self> = Box::leak(cld).into();
        cld_map::register(ptr);
        ptr
    }

    /// 从 `&MSBox<Klass>` 通过 Deref 获取 `NonNull<Klass>`，不转移所有权。
    fn klass_ptr(mb: &MSBox<Klass>) -> NonNull<Klass> {
        let r: &Klass = mb;
        unsafe { NonNull::new_unchecked(r as *const Klass as *mut Klass) }
    }

    // ── findLoadedClass ─────────────────────────────────────────────────

    pub fn find_loaded_class(&self, name: &SymbolHandle) -> Option<NonNull<Klass>> {
        match self.klasses.get(name)?.value() {
            ClassEntry::Done { klass, .. } => Some(Self::klass_ptr(klass)),
            ClassEntry::Loading => None,
        }
    }

    // ── defineClass ─────────────────────────────────────────────────────

    pub fn define_class(&self, name: &str, bytes: &[u8]) -> LoadResult<NonNull<Klass>> {
        let sym = SymbolTable::intern(name.into());

        if let Some(entry) = self.klasses.get(&sym) {
            match entry.value() {
                ClassEntry::Done { klass, .. } => return Ok(Self::klass_ptr(klass)),
                ClassEntry::Loading => {
                    drop(entry);
                    return self.wait_for(&sym);
                }
            }
        }

        match self.klasses.entry(sym.clone()) {
            Entry::Occupied(e) => {
                return match e.get() {
                    ClassEntry::Done { klass, .. } => Ok(Self::klass_ptr(klass)),
                    ClassEntry::Loading => {
                        drop(e);
                        self.wait_for(&sym)
                    }
                };
            }
            Entry::Vacant(e) => {
                e.insert(ClassEntry::Loading);
            }
        }

        let result = self.build_klass(bytes);

        match result {
            Ok((klass_box, super_entry)) => {
                let ptr = Self::klass_ptr(&klass_box);
                self.klasses.insert(
                    sym.clone(),
                    ClassEntry::Done {
                        klass: klass_box,
                        super_entry,
                    },
                );
                self.wake(&sym);
                Ok(ptr)
            }
            Err(e) => {
                self.klasses.remove(&sym);
                self.wake(&sym);
                Err(e)
            }
        }
    }

    // ── loadClass ───────────────────────────────────────────────────────

    pub fn load_class(&self, name: &str) -> LoadResult<NonNull<Klass>> {
        unimplemented!()
    }

    // ── linkClass ───────────────────────────────────────────────────────

    pub fn link_class(&self, name: &str) -> LoadResult<()> {
        let sym = SymbolTable::intern(name.into());
        let entry = self
            .klasses
            .get(&sym)
            .ok_or_else(|| LoadError::NotLoaded(name.into()))?;

        let (klass_ptr, super_entry) = match entry.value() {
            ClassEntry::Done { klass, super_entry } => (Self::klass_ptr(klass), *super_entry),
            ClassEntry::Loading => return Err(LoadError::StillLoading(name.into())),
        };

        let normal = match unsafe { &*klass_ptr.as_ptr() } {
            Klass::Normal(n) => n,
            _ => return Ok(()),
        };

        if normal.super_klass.is_some() {
            return Ok(());
        }

        let se = match super_entry {
            Some(e) => e,
            None => return Ok(()),
        };

        let super_name = unsafe { se.as_ref().name.utf8().clone() };
        let super_klass = self.load_class(&super_name)?;

        let super_normal = match unsafe { &*super_klass.as_ptr() } {
            Klass::Normal(n) => unsafe {
                NonNull::new_unchecked(n as *const NormalKlass as *mut NormalKlass)
            },
            _ => return Err(LoadError::SuperNotNormal(super_name)),
        };

        unsafe {
            let nm = &mut *(klass_ptr.as_ptr() as *mut NormalKlass);
            nm.super_klass = Some(super_normal);
        }

        Ok(())
    }

    // ── 内部 ───────────────────────────────────────────────────────────

    fn build_klass(
        &self,
        bytes: &[u8],
    ) -> LoadResult<(MSBox<Klass>, Option<NonNull<ClassCPEntry>>)> {
        let cf = ClassFile::from(bytes).map_err(LoadError::Parse)?;
        let (nk, super_entry) =
            NormalKlass::from(cf, NonNull::from(self)).map_err(LoadError::Resolve)?;
        let klass_box = MSBox::new(&self.ms_allocator, Klass::Normal(nk));
        Ok((klass_box, super_entry))
    }

    fn wait_for(&self, sym: &SymbolHandle) -> LoadResult<NonNull<Klass>> {
        self.waiters
            .entry(sym.clone())
            .or_default()
            .push(thread::current());
        loop {
            thread::park();
            if let Some(entry) = self.klasses.get(sym) {
                if let ClassEntry::Done { klass, .. } = entry.value() {
                    return Ok(Self::klass_ptr(klass));
                }
            }
            return Err(LoadError::LoadingFailed(sym.utf8().clone()));
        }
    }

    fn wake(&self, sym: &SymbolHandle) {
        if let Some((_, waiters)) = self.waiters.remove(sym) {
            for t in waiters {
                t.unpark();
            }
        }
    }
}

// ── 测试 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_loader::{class_path::MemClassPath, ms_box::ms_init};
    use crate::oops::oop_handle::init_oop_storages;
    use std::collections::HashMap;
    use std::sync::Once;

    fn vm_init() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            assert!(unsafe { ms_init() });
            unsafe { init_oop_storages() };
        });
    }

    fn bytes() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&0xCAFEBABEu32.to_be_bytes());
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x34]);
        b.extend_from_slice(&17u16.to_be_bytes());
        b.extend_from_slice(&[10, 0x00, 0x03, 0x00, 0x0D]);
        b.extend_from_slice(&[7, 0x00, 0x0E]);
        b.extend_from_slice(&[7, 0x00, 0x0F]);
        b.extend_from_slice(&[1, 0x00, 0x06]);
        b.extend_from_slice(b"<init>");
        b.extend_from_slice(&[1, 0x00, 0x03]);
        b.extend_from_slice(b"()V");
        b.extend_from_slice(&[1, 0x00, 0x04]);
        b.extend_from_slice(b"Code");
        b.extend_from_slice(&[1, 0x00, 0x0F]);
        b.extend_from_slice(b"LineNumberTable");
        b.extend_from_slice(&[1, 0x00, 0x12]);
        b.extend_from_slice(b"LocalVariableTable");
        b.extend_from_slice(&[1, 0x00, 0x04]);
        b.extend_from_slice(b"this");
        b.extend_from_slice(&[1, 0x00, 0x10]);
        b.extend_from_slice(b"Lpkg/TestSimple;");
        b.extend_from_slice(&[1, 0x00, 0x0A]);
        b.extend_from_slice(b"SourceFile");
        b.extend_from_slice(&[1, 0x00, 0x0F]);
        b.extend_from_slice(b"TestSimple.java");
        b.extend_from_slice(&[12, 0x00, 0x04, 0x00, 0x05]);
        b.extend_from_slice(&[1, 0x00, 0x0E]);
        b.extend_from_slice(b"pkg/TestSimple");
        b.extend_from_slice(&[1, 0x00, 0x10]);
        b.extend_from_slice(b"java/lang/Object");
        b.extend_from_slice(&[1, 0x00, 0x0F]);
        b.extend_from_slice(b"TestSimple.java");
        b.extend_from_slice(&[0x00, 0x21, 0x00, 0x02, 0x00, 0x03]);
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        b.extend_from_slice(&0x00_01u16.to_be_bytes());
        b.extend_from_slice(&[0x00, 0x01, 0x00, 0x04, 0x00, 0x05]);
        b.extend_from_slice(&[0x00, 0x01, 0x00, 0x06]);
        let pos = b.len();
        b.extend_from_slice(&0u32.to_be_bytes());
        b.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);
        b.extend_from_slice(&5u32.to_be_bytes());
        b.extend_from_slice(&[0x2A, 0xB7, 0x00, 0x01, 0xB1]);
        b.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let end = b.len();
        b[pos..pos + 4].copy_from_slice(&((end - pos - 4) as u32).to_be_bytes());
        b.extend_from_slice(&[0x00, 0x00]);
        b
    }

    #[test]
    fn define_and_find() {
        vm_init();
        let mut mem = HashMap::new();
        mem.insert("pkg/TestSimple".into(), bytes());
        let cld = ClassLoaderData::new(Some("t".into()), Box::new(MemClassPath::new(mem)));
        let r = unsafe { cld.as_ptr().as_ref().unwrap() };

        let sym = SymbolTable::intern("pkg/TestSimple".into());
        assert!(r.find_loaded_class(&sym).is_none());

        let b = bytes();
        let k1 = r.define_class("pkg/TestSimple", &b).unwrap();
        assert_eq!(k1, r.find_loaded_class(&sym).unwrap());
        assert_eq!(k1, r.define_class("pkg/TestSimple", &b).unwrap());

        unsafe {
            drop(Box::from_raw(cld.as_ptr()));
        }
    }

    #[test]
    fn load_via_class_path() {
        vm_init();
        let mut mem = HashMap::new();
        mem.insert("pkg/TestSimple".into(), bytes());
        let cld = ClassLoaderData::new(Some("t".into()), Box::new(MemClassPath::new(mem)));
        let r = unsafe { cld.as_ptr().as_ref().unwrap() };

        let k = r.load_class("pkg/TestSimple").unwrap();
        let sym = SymbolTable::intern("pkg/TestSimple".into());
        assert_eq!(k, r.find_loaded_class(&sym).unwrap());
        assert!(r.load_class("pkg/NotFound").is_err());

        unsafe {
            drop(Box::from_raw(cld.as_ptr()));
        }
    }
}
