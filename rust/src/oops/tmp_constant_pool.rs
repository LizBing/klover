use std::collections::{HashMap, hash_map::Entry};

use crate::{class_parser::MethodMemRef, oops::constant_entries::MethodRef, runtime::ms_api::{MsAllocator, MsBox}};

#[derive(Debug, Clone)]
pub enum TmpConstantPoolEntry<'metadata> {
    MethodRef(&'metadata MethodRef<'metadata>),
    FieldRef,
}

pub struct TmpConstantPool<'metadata> {
    table: HashMap<usize, TmpConstantPoolEntry<'metadata>>,
}

pub trait IntoTmpCPEntry<'metadata>: Copy {
    fn into_key(self) -> usize;
    fn into_value(self, msa: &'metadata MsAllocator) -> TmpConstantPoolEntry<'metadata>;
}

impl<'metadata> IntoTmpCPEntry<'metadata> for &MethodMemRef {
    fn into_key(self) -> usize {
        self as *const _ as _
    }

    fn into_value(self, msa: &'metadata MsAllocator) -> TmpConstantPoolEntry<'metadata> {
        let mref = MethodRef::build(self);
        TmpConstantPoolEntry::MethodRef(MsBox::new(msa, mref).leak())
    }
}

impl TmpConstantPool<'_> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
}

impl<'metadata> TmpConstantPool<'metadata> {
    pub fn resolve<T>(&mut self, msa: &'metadata MsAllocator, into_entry: T) -> TmpConstantPoolEntry<'metadata>
    where
        T: IntoTmpCPEntry<'metadata>
    {
        let key = into_entry.into_key();
        match self.table.entry(key) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                entry.insert(into_entry.into_value(msa)).clone()
            }
        }
    }
}
