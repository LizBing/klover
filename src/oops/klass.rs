/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use cafebabe::ClassFile;
use cafebabe::descriptors::ClassName;
use crate::class_data::{class_loader};
use crate::class_data::java_classes::{JavaLangObject};
use crate::oops::field::{Fields};
use crate::oops::obj_desc::ObjDesc;
use crate::oops::obj_handle::ObjHandle;
use crate::oops::oop::ObjPtr;

#[derive(Debug)]
pub struct NormalKlass<'a> {
    _name: ClassName<'a>,
    _super: Option<&'a NormalKlass<'a>>,
    _loader: ObjHandle,

    // leaked memory
    _metadata: &'a [u8],
    _class_file: &'a ClassFile<'a>,

    _mirror: ObjHandle,

    _fields: Fields<'a>,

    // hot fields
    _cp_entries: usize,
}

impl NormalKlass<'static> {
    // Returning false means ClassNotFoundException.
    pub fn new(
        loader: ObjPtr,
        metadata: Vec<u8>,
    ) -> Option<Self> {
        let leaked = metadata.leak();

        let cf = Box::leak(Box::new(match cafebabe::parse_class(leaked) {
            Ok(n) => n,
            Err(_) => return None,
        }));

        let name = cf.this_class.clone();

        let super_class = match cf.super_class.clone() {
            Some(s) => Some(class_loader::load_normal_class(loader, s.to_string())?),
            None => None    // invariant: loading of java.lang.Object
        };

        // todo: resolve fields and methods.
        let offs = match super_class {
            Some(s) => s.size_of_instance(),
            None => ObjDesc::size_of_normal_desc()
        };
        let fields = Fields::new(offs, &cf.fields);

        Some(Self {
            _name: name,
            _super: super_class,
            _loader: ObjHandle::with_oop(loader),
            _metadata: leaked,
            _class_file: cf,
            _mirror: ObjHandle::new(),
            _fields: fields,
            _cp_entries: cf.constantpool_iter().count()
        })
    }
}

unsafe impl Send for NormalKlass<'_> {}
unsafe impl Sync for NormalKlass<'_> {}

impl Drop for NormalKlass<'_> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self._metadata as *const _ as *mut [u8]));
            drop(Box::from_raw(self._class_file as *const _ as *mut ClassFile));
        }
    }
}

impl NormalKlass<'_> {
    pub fn size_of_instance(&self) -> usize {
        self._fields.size_of_instance
    }

    pub fn size_of_mirror(&self) -> usize {
        self._fields.size_of_statics
    }

    pub fn cp_entries(&self) -> usize {
        self._cp_entries
    }
}

pub struct PrimitiveKlass {
    _name: String,
    _mirror: ObjHandle,
    _size_of_instance: usize
}

pub struct ArrayKlass<'a> {
    _name: String,
    _mirror: ObjHandle,
    _dimensions: usize,
    _elem_type: &'a Klass<'a>
}

pub enum Klass<'a> {
    Normal(NormalKlass<'a>),
    Primitive(PrimitiveKlass),
    Array(ArrayKlass<'a>)
}

impl<'a> Klass<'a> {
    pub fn as_normal(&self) -> &'a NormalKlass {
        match self {
            Self::Normal(n) => n,
            _ => panic!()
        }
    }
}

impl<'a> Klass<'a> {
    pub fn name(&self) -> &str {
        match self {
            Self::Normal(n) => n._name.as_ref(),
            Self::Primitive(n) => n._name.as_str(),
            Self::Array(n) => n._name.as_str()
        }
    }

    pub fn mirror(&self) -> &ObjHandle {
        match self {
            Self::Normal(n) => &n._mirror,
            Self::Primitive(n) => &n._mirror,
            Self::Array(n) => &n._mirror,
        }
    }

    pub fn super_class(&self) -> Option<&'static NormalKlass> {
        match self {
            Self::Normal(n) => n._super,
            _ => Some(JavaLangObject::this().as_normal())
        }
    }

    pub fn size_of_instance(&self) -> Option<usize> {
        match self {
            Self::Normal(n) => Some(n.size_of_instance()),
            Self::Primitive(n) => Some(n._size_of_instance),
            Self::Array(_) => None
        }
    }

    pub fn size_of_mirror(&self) -> usize {
        match self {
            Self::Normal(n) => n.size_of_mirror(),
            _ => JavaLangObject::size_of_instance()
        }
    }

    pub fn is_array_klass(&self) -> bool {
        if let Self::Array(_) = self {
            true
        } else {
            false
        }
    }
}
