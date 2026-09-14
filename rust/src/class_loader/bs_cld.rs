use std::{collections::HashMap, marker::PhantomData, ops::Deref, path::Path, sync::LazyLock};

use dashmap::{DashMap, DashSet};

use crate::{class_loader::{class_path::ClassPath, class_slot::ClassSlot, load_error::{LoadError, LoadErrorKind, LoadResult}, ms_api::{MSAllocator, MSBox, MSRef}}, oops::{klass::Klass, normal_klass::{NormalKlass, UnlinkedNormalKlass}, symbol_table::{SymbolHandle, SymbolTable}}};

pub struct BootstrapCLD;

static MSA: MSAllocator = MSAllocator::new();
static CLASS_TABLE: LazyLock<DashMap<SymbolHandle, ClassSlot>> = LazyLock::new(|| DashMap::new());

impl BootstrapCLD {
    pub fn ms_allocator() -> &'static MSAllocator {
        &MSA
    }

    pub fn make_load_error(klass_name: &SymbolHandle, kind: LoadErrorKind) -> LoadError {
        LoadError {
            __: PhantomData,
            cld_name: Some("Klover Bootstrap Class Loader".into()),
            klass_name: klass_name.utf8().into(),
            kind,
        }
    }
}

impl BootstrapCLD {
    pub fn find_class(name: &str) -> LoadResult<MSRef<Klass>> {
        let sh = SymbolTable::intern(name);
        Self::find_class_sym(&sh)
    }
    
    pub fn find_class_sym(name: &SymbolHandle) -> LoadResult<MSRef<Klass>> {
        if let Some(klass) = Self::find_loaded_class(name) {
            return Ok(klass);
        }

        let bytes = match ClassPath::read_bs_class(name.utf8()) {
            Some(b) => b,
            None => return Err(Self::make_load_error(name, LoadErrorKind::NotFound)),
        };

        Self::define_class(name, &bytes)
    }

    fn find_loaded_class(name: &SymbolHandle) -> Option<MSRef<Klass>> {
        CLASS_TABLE.get(name)
            .map(|r| MSRef::from(&r.klass))
    }

    fn define_class(name: &SymbolHandle, bytes: &[u8]) -> LoadResult<MSRef<Klass>> {
        let mut parse_option = cafebabe::ParseOptions::default();
        parse_option.parse_bytecode(true);

        let cf = cafebabe::parse_class_with_options(bytes, &parse_option)
            .map_err(|e| Self::make_load_error(name, LoadErrorKind::Parse(e)))?;

        let unlinked = UnlinkedNormalKlass::build(cf, None);
        let normal = NormalKlass::try_from(unlinked)
            .map_err(|e| Self::make_load_error(name, LoadErrorKind::Linkage(e)))?;

        let boxed = MSBox::new(Self::ms_allocator(), Klass::Normal(normal));
        let res = MSRef::from(&boxed);
        
        match CLASS_TABLE.insert(name.clone(), ClassSlot::new(boxed)) {
            Some(_) => Err(Self::make_load_error(name, LoadErrorKind::Duplicated)),
            None => Ok(res),
        }
    }
}
