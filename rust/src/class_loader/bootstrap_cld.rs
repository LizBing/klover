use std::{ops::Deref, ptr::NonNull, sync::{LazyLock, OnceLock}};

use dashmap::{DashMap, Entry};

use crate::{
    class_loader::{class_path::ClassPath, load_error::{LoadError, LoadResult}, ms_box::{MSAllocator, MSBox}}, class_parser::class_file::ClassFile, oops::{array_klass::ArrayKlass, desc::{FieldDesc, FieldElemType}, klass::Klass, normal_klass::NormalKlass, oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, prim_klass::PrimKlass, symbol_table::{SymbolHandle, SymbolTable}}
};

pub struct BootstrapCLD {
    msa: MSAllocator,
    klasses: LazyLock<DashMap<SymbolHandle, MSBox<Klass>>>,

    boolean_klass: OnceLock<MSBox<Klass>>,
    byte_klass: OnceLock<MSBox<Klass>>,
    char_klass: OnceLock<MSBox<Klass>>,
    double_klass: OnceLock<MSBox<Klass>>,
    float_klass: OnceLock<MSBox<Klass>>,
    int_klass: OnceLock<MSBox<Klass>>,
    long_klass: OnceLock<MSBox<Klass>>,
    short_klass: OnceLock<MSBox<Klass>>,
}

unsafe impl Sync for BootstrapCLD {}

static BSCLD: BootstrapCLD = BootstrapCLD {
    msa: MSAllocator::new(),
    klasses: LazyLock::new(|| DashMap::new()),
    
    boolean_klass: OnceLock::new(),
    byte_klass: OnceLock::new(),
    char_klass: OnceLock::new(),
    double_klass: OnceLock::new(),
    float_klass: OnceLock::new(),
    int_klass: OnceLock::new(),
    long_klass: OnceLock::new(),
    short_klass: OnceLock::new(),
};

impl BootstrapCLD {
    pub fn bs_msa() -> &'static MSAllocator {
        &BSCLD.msa
    }
}

impl BootstrapCLD {
    pub fn find_class(name: &str) -> LoadResult<NonNull<Klass>> {
        if let Some(x) = name.chars().next() {
            if x == '[' {
                return Self::find_array_klass(name)
            }
        }
        
        if let Some(x) = Self::find_prim_klass(name) {
            return Ok(x)
        }

        Self::find_normal_klass(name)
    }
    
    fn find_prim_klass(name: &str) -> Option<NonNull<Klass>> {
        let boxed = match name {
            "boolean" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<bool>())))),
            "byte" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<i8>())))),
            "char" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<u16>())))),
            "double" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<f64>())))),
            "float" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<f32>())))),
            "int" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<i32>())))),
            "long" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<i64>())))),
            "short" => BSCLD.int_klass.get_or_init(|| MSBox::new(&BSCLD.msa, Klass::Primitive(PrimKlass::new(name, size_of::<i16>())))),

            _ => return None,
        };

        unsafe { Some(NonNull::new_unchecked(boxed.deref() as *const Klass as _)) }
    }
    
    fn find_array_klass(name: &str) -> LoadResult<NonNull<Klass>> {
        let sym = SymbolTable::intern(name);
        let entry = BSCLD.klasses.entry(sym);

        let vacant = match entry {
            Entry::Occupied(x) => return unsafe { Ok(NonNull::new_unchecked(x.get().deref() as *const Klass as *mut Klass)) },
            Entry::Vacant(v) => v
        };
        
        let desc = match FieldDesc::from(name) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Resolve(e))
        };

        let klass = ArrayKlass {
            name: name.into(),
            desc,
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID)
        };

        let boxed = MSBox::new(&BSCLD.msa, Klass::Array(klass));
        let r = vacant.insert(boxed);

        return unsafe { Ok(NonNull::new_unchecked(r.deref().deref() as *const Klass as *mut Klass)) };
    }
    
    fn find_normal_klass(name: &str) -> LoadResult<NonNull<Klass>> {
        let sym = SymbolTable::intern(name);
        let entry = BSCLD.klasses.entry(sym);

        let vacant = match entry {
            Entry::Occupied(x) => return unsafe { Ok(NonNull::new_unchecked(x.get().deref() as *const Klass as *mut Klass)) },
            Entry::Vacant(v) => v
        };

        let bytes = match ClassPath::read_bs_class(name) {
            Some(x) => x,
            None => return Err(LoadError::NotFound(name.into()))
        };

        let cf = match ClassFile::from(&bytes) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Parse(e))
        };

        let klass = match NormalKlass::from(cf, &BSCLD.msa) {
            Ok(x) => x,
            Err(e) => return Err(LoadError::Resolve(e))
        };
        klass.cld.set(None).unwrap();

        let boxed = MSBox::new(&BSCLD.msa, Klass::Normal(klass));
        let r = vacant.insert(boxed);

        return unsafe { Ok(NonNull::new_unchecked(r.deref().deref() as *const Klass as *mut Klass)) };
    }
}
