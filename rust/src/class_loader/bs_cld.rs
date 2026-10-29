use std::sync::LazyLock;

use dashmap::DashMap;

use crate::{
    class_loader::{
        class_path::ClassPath,
        load_error::{LoadError, LoadErrorKind, LoadResult},
        oop_storage::OOPStorage,
    },
    class_parser::parse_class,
    oops::{
        klass::Klass,
        normal_klass::{NormalKlass, UnlinkedNormalKlass},
        symbol_table::SymbolHandle,
    },
    runtime::ms_api::{MsAllocator, MsBox},
};

pub struct BootstrapCLD;

static MSA: MsAllocator = MsAllocator::new();
static OOP_STORAGE: OOPStorage = OOPStorage::new(&MSA);
static CLASS_TABLE: LazyLock<DashMap<SymbolHandle, &Klass>> = LazyLock::new(|| DashMap::new());

impl BootstrapCLD {
    pub fn ms_allocator() -> &'static MsAllocator {
        &MSA
    }

    pub fn oop_storage() -> &'static OOPStorage<'static> {
        &OOP_STORAGE
    }

    fn make_load_error(klass_name: &SymbolHandle, kind: LoadErrorKind) -> LoadError {
        LoadError::new(
            Some("Klover Bootstrap Class Loader".into()),
            klass_name.utf8().into(),
            kind,
        )
    }
}

impl BootstrapCLD {
    pub fn find_klass(name: &SymbolHandle) -> LoadResult<&'static Klass<'static>> {
        if let Some(klass) = Self::find_loaded_class(name) {
            return Ok(klass);
        }

        let bytes = match ClassPath::read_bs_class(name.utf8()) {
            Some(b) => b,
            None => return Err(Self::make_load_error(name, LoadErrorKind::NotFound)),
        };

        Self::define_class(name, &bytes)
    }

    pub fn find_normal_klass(name: &SymbolHandle) -> LoadResult<&'static NormalKlass<'static>> {
        // Normal is currently the only supported Klass variant.
        Ok(Self::find_klass(name)?.as_normal().unwrap())
    }

    fn find_loaded_class(name: &SymbolHandle) -> Option<&'static Klass<'static>> {
        CLASS_TABLE.get(name).map(|r| *r)
    }

    fn define_class(name: &SymbolHandle, bytes: &[u8]) -> LoadResult<&'static Klass<'static>> {
        let cf =
            parse_class(bytes).map_err(|e| Self::make_load_error(name, LoadErrorKind::Parse(e)))?;

        let unlinked = UnlinkedNormalKlass::build(cf, None);

        let super_name = unlinked.super_name.as_ref();
        let super_klass = match super_name {
            Some(n) => Some(Self::find_normal_klass(n)?),
            None => None,
        };

        let mut interfaces = MsBox::new_uninit_slice(&MSA, unlinked.interfaces.len());
        for (slot, interface_name) in interfaces.iter_mut().zip(unlinked.interfaces.iter()) {
            slot.write(Self::find_normal_klass(interface_name)?);
        }
        // Every slot was initialized successfully; failed loads return before linking.
        let interfaces = unsafe { interfaces.assume_init() };

        let normal = NormalKlass::link(unlinked, super_klass, interfaces)
            .map_err(|e| Self::make_load_error(name, LoadErrorKind::Linkage(e)))?;

        let res = MsBox::new(&MSA, Klass::Normal(normal)).leak();

        if CLASS_TABLE.insert(name.clone(), res).is_some() {
            return Err(Self::make_load_error(name, LoadErrorKind::Duplicated));
        }

        Ok(res)
    }
}
