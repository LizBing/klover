use std::sync::LazyLock;

use dashmap::{DashMap, Entry};

use crate::{class_loader::{class_path::ClassPath, class_slot::ClassSlot, load_error::{LoadError, LoadErrorKind, LoadResult}, oop_storage::OOPStorage}, oops::{klass::Klass, normal_klass::{NormalKlass, UnlinkedNormalKlass}, symbol_table::SymbolHandle}, runtime::ms_api::{MsAllocator, MsBox}};

pub struct BootstrapCLD;

static MSA: MsAllocator = MsAllocator::new();
static OOP_STORAGE: OOPStorage<'static> = OOPStorage::new(&MSA);
static CLASS_TABLE: LazyLock<DashMap<SymbolHandle, ClassSlot<'static>>> = LazyLock::new(|| DashMap::new());

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
    pub fn find_class(name: &SymbolHandle) -> LoadResult<&'static Klass<'static>> {
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
        Ok(Self::find_class(name)?.as_normal_klass().unwrap())
    }

    fn find_loaded_class(name: &SymbolHandle) -> Option<&'static Klass<'static>> {
        CLASS_TABLE.get(name)
            .map(|r| r.klass() )
    }

    fn define_class(name: &SymbolHandle, bytes: &[u8]) -> LoadResult<&'static Klass<'static>> {
        let mut parse_option = cafebabe::ParseOptions::default();
        parse_option.parse_bytecode(true);

        let cf = cafebabe::parse_class_with_options(bytes, &parse_option)
            .map_err(|e| Self::make_load_error(name, LoadErrorKind::Parse(e)))?;

        let unlinked = UnlinkedNormalKlass::build(cf, None);
        let normal = NormalKlass::try_from(unlinked)
            .map_err(|e| Self::make_load_error(name, LoadErrorKind::Linkage(e)))?;

        let boxed = MsBox::new(&MSA, Klass::Normal(normal));

        match CLASS_TABLE.entry(name.clone()) {
            Entry::Occupied(_) => return Err(Self::make_load_error(name, LoadErrorKind::Duplicated)),
            Entry::Vacant(v) => {
                let klass = boxed.leak();
                v.insert(ClassSlot::new(klass)).downgrade();
                Ok(klass)
            }
        }
    }
}
