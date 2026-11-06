use dashmap::{DashMap, mapref::entry::Entry};

use crate::{
    class_loader::{
        class_path::ClassPath,
        load_error::{LoadError, LoadResult},
        oop_storage::{OopHandle, OopStorage, WeakOopSlotId},
    },
    class_parser::{self, ClassImage},
    oops::{
        klass::Klass,
        normal_klass::{NormalKlass, UnlinkedNormalKlass},
        oops_errors::LinkageError,
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::ms_api::{MsAllocator, MsBox},
};

#[derive(Debug)]
struct KlassPtr(usize);

#[derive(Debug)]
pub struct ClassLoaderData {
    name: Option<String>,

    msa: &'static MsAllocator,
    pub storage: OopStorage,
    mirror_id: WeakOopSlotId, // java.lang.ClassLoader

    table: DashMap<SymbolHandle, KlassPtr>,
}

impl Drop for ClassLoaderData {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.msa as *const MsAllocator as *mut MsAllocator);
        }
    }
}

impl ClassLoaderData {
    pub fn new(name: Option<String>) -> Self {
        let msa = Box::leak(Box::new(MsAllocator::new()));
        let storage = OopStorage::new(msa);
        let mirror = storage.create_weak_id();

        Self {
            name,
            msa,
            storage,
            mirror_id: mirror,
            table: DashMap::new(),
        }
    }

    pub fn oop_storage(&self) -> &OopStorage {
        &self.storage
    }

    pub fn ms_allocator(&self) -> &MsAllocator {
        self.msa
    }

    fn mirror(&self) -> OopHandle<'_> {
        self.storage.get_weak(self.mirror_id)
    }
}

impl ClassLoaderData {
    pub fn find_loaded_klass<'this>(
        &'this self,
        name: &SymbolHandle,
    ) -> Option<&'this Klass<'this>> {
        self.table.get(name).map(|entry| {
            let KlassPtr(ptr) = *entry.value();
            // Entries are immutable, fully linked arena allocations. Their
            // lifetime is bounded by this loader's allocator.
            unsafe { &*(ptr as *const Klass) }
        })
    }

    pub fn load_klass<'this>(
        &'this self,
        klass_name: &SymbolHandle,
    ) -> LoadResult<&'this Klass<'this>> {
        self.load_klass_inner(klass_name, &mut Vec::new())
    }

    fn load_klass_inner<'this>(
        &'this self,
        klass_name: &SymbolHandle,
        loading: &mut Vec<SymbolHandle>,
    ) -> LoadResult<&'this Klass<'this>> {
        if self.mirror().slot.load::<false>().is_null() {
            if let Some(klass) = self.find_loaded_klass(klass_name) {
                return Ok(klass);
            }

            let Some(bytes) = ClassPath::read_bs_class(klass_name.utf8()) else {
                return Err(LoadError::NotFound);
            };

            let image = class_parser::parse_class(&bytes).map_err(LoadError::Parse)?;
            if image.this_class_name() != klass_name.utf8() {
                return Err(LoadError::NameMismatch {
                    expected: klass_name.utf8().into(),
                    actual: image.this_class_name().into(),
                });
            }

            match self.define_image(image, loading) {
                // Another load/definition may have published the same class
                // while we loaded its dependencies. Loads reuse that identity;
                // explicit definitions still report a duplicate.
                Err(LoadError::Duplicated) => Ok(self.find_loaded_klass(klass_name).unwrap()),
                result => result,
            }
        } else {
            unreachable!("Class loader unsupported")
        }
    }

    pub fn define_klass<'this>(&'this self, bytes: &[u8]) -> LoadResult<&'this Klass<'this>> {
        let image = class_parser::parse_class(bytes).map_err(LoadError::Parse)?;
        self.define_image(image, &mut Vec::new())
    }

    fn define_image<'this>(
        &'this self,
        image: ClassImage,
        loading: &mut Vec<SymbolHandle>,
    ) -> LoadResult<&'this Klass<'this>> {
        let name = SymbolTable::intern(image.this_class_name());
        if self.find_loaded_klass(&name).is_some() {
            return Err(LoadError::Duplicated);
        }
        if loading.contains(&name) {
            return Err(LoadError::Linkage(LinkageError::ClassCircularity {
                name: name.utf8().into(),
            }));
        }

        // Track only the current recursive dependency chain, so a shared
        // ancestor is allowed and failed loads leave no persistent state.
        loading.push(name);
        let result = self.link_and_publish(image, loading);
        loading.pop();
        result
    }

    fn link_and_publish<'this>(
        &'this self,
        image: ClassImage,
        loading: &mut Vec<SymbolHandle>,
    ) -> LoadResult<&'this Klass<'this>> {
        let unlinked = UnlinkedNormalKlass::build(image, self);
        let super_klass = match &unlinked.super_name {
            Some(name) => Some(self.load_normal_dependency(name, loading)?),
            None => None,
        };
        let mut interfaces =
            MsBox::new_uninit_slice(self.ms_allocator(), unlinked.interfaces.len());
        for (slot, name) in interfaces.iter_mut().zip(unlinked.interfaces.iter()) {
            slot.write(self.load_normal_dependency(name, loading)?);
        }
        // Every slot above contains a valid, loader-lived reference. On an
        // early error, the MaybeUninit slice owns no resources to destroy.
        let interfaces = unsafe { interfaces.assume_init() };
        let klass =
            NormalKlass::link(unlinked, super_klass, interfaces).map_err(LoadError::Linkage)?;

        // Never hold a table entry guard across recursive dependency loading.
        match self.table.entry(klass.name().clone()) {
            Entry::Occupied(_) => Err(LoadError::Duplicated),
            Entry::Vacant(entry) => {
                let klass = MsBox::new(self.ms_allocator(), Klass::Normal(klass)).leak();
                entry.insert(KlassPtr(klass as *const Klass<'this> as usize));
                Ok(klass)
            }
        }
    }

    fn load_normal_dependency<'this>(
        &'this self,
        name: &SymbolHandle,
        loading: &mut Vec<SymbolHandle>,
    ) -> LoadResult<&'this NormalKlass<'this>> {
        let klass = self
            .load_klass_inner(name, loading)
            .map_err(|error| match error {
                LoadError::NotFound => LoadError::NoClassDefFound,
                error => error,
            })?;
        klass.as_normal().ok_or_else(|| {
            LoadError::Linkage(LinkageError::NotNormalKlass {
                name: name.utf8().into(),
            })
        })
    }
}
