use std::sync::{Arc, LazyLock, OnceLock};

use dashmap::{DashMap, Entry};

use crate::{
    class_loader::{
        class_path::ClassPath, class_slot::{ClassLoadState, ClassSlot}, load_error::{LoadError, LoadResult}, ms_api::{MSAllocator, MSBox, MSRef},
    }, class_parser::class_file::ClassFile, gc_bindings::oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, oops::{
        array_klass::ArrayKlass,
        desc::FieldDesc,
        klass::Klass,
        normal_klass::{NormalKlass, UnlinkedNormalKlass},
        prim_klass::PrimKlass,
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

pub struct BootstrapCLD {
    msa: MSAllocator,
    klasses: LazyLock<DashMap<SymbolHandle, Arc<ClassSlot>>>,

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
    pub fn find_class(name: &str) -> LoadResult<MSRef<Klass>> {
        if let Some(x) = Self::find_prim_klass(name) {
            return Ok(x);
        }

        let sym = SymbolTable::intern(name);
        let (slot, is_leader) = match BSCLD.klasses.entry(sym.clone()) {
            Entry::Occupied(x) => {
                let slot = x.get().clone();
                (slot, false)
            }

            Entry::Vacant(x) => {
                let slot = Arc::new(ClassSlot::default());
                x.insert(slot.clone());

                (slot, true)
            }
        };

        let load_res = if is_leader {
            if name.starts_with('[') {
                Self::find_array_klass(sym)
            } else {
                Self::find_normal_klass(sym)
            }
        } else {
            let mut guard = slot.state.lock();
            loop {
                match &*guard {
                    ClassLoadState::Loading { owner } => {
                        if owner.eq(&std::thread::current().id()) {
                            return Err(LoadError::Circularity)
                        }
                        slot.completed.wait(&mut guard);
                    }

                    ClassLoadState::Loaded(klass) => {
                        return Ok(klass.into());
                    }

                    ClassLoadState::Failed(e) => return Err(e.clone())
                }
            }
        };

        match load_res {
            Ok(x) => {
                let res = (&x).into();

                {
                    let mut state = slot.state.lock();
                    *state = ClassLoadState::Loaded(x);
                }
                slot.completed.notify_all();

                return Ok(res);
            }

            Err(e) => {
                {
                    let mut state = slot.state.lock();
                    *state = ClassLoadState::Failed(e.clone());
                }
                slot.completed.notify_all();

                return Err(e)
            }
        }
    }

    fn find_prim_klass(name: &str) -> Option<MSRef<Klass>> {
        let boxed = match name {
            "boolean" => BSCLD.boolean_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<bool>())),
                )
            }),
            "byte" => BSCLD.byte_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<i8>())),
                )
            }),
            "char" => BSCLD.char_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<u16>())),
                )
            }),
            "double" => BSCLD.double_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<f64>())),
                )
            }),
            "float" => BSCLD.float_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<f32>())),
                )
            }),
            "int" => BSCLD.int_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<i32>())),
                )
            }),
            "long" => BSCLD.long_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<i64>())),
                )
            }),
            "short" => BSCLD.short_klass.get_or_init(|| {
                MSBox::new(
                    &BSCLD.msa,
                    Klass::Primitive(PrimKlass::new(name, size_of::<i16>())),
                )
            }),

            _ => return None,
        };

        Some(boxed.into())
    }

    fn find_array_klass(sym: SymbolHandle) -> LoadResult<MSBox<Klass>> {
        let desc = FieldDesc::from(sym.utf8())?;

        let klass = Klass::Array(ArrayKlass {
            name: sym,
            desc,
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID),
        });

        let boxed = MSBox::new(Self::bs_msa(), klass);

        Ok(boxed)
    }

    fn find_normal_klass(sym: SymbolHandle) -> LoadResult<MSBox<Klass>> {
        let bytes = match ClassPath::read_bs_class(sym.utf8()) {
            Some(x) => x,
            None => return Err(LoadError::NotFound(sym.utf8().into())),
        };

        let cf = ClassFile::from(&bytes)?;
        let unlinked = UnlinkedNormalKlass::build(cf, None)?;
        let boxed = NormalKlass::link(unlinked, None)?;

        Ok(boxed)
    }
}
