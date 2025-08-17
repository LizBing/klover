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
use crate::class_data::java_classes::{JavaLangClass, JavaLangObject};
use crate::common::universe;
use crate::oops::field::{Field, Fields};
use crate::oops::obj_desc::ObjDesc;
use crate::oops::obj_handle::ObjHandle;
use crate::oops::oop::ObjPtr;

#[derive(Debug)]
pub struct Klass<'a> {
    _name: Option<ClassName<'a>>,
    _super: Option<&'a Klass<'a>>,
    _loader: ObjHandle,

    _metadata: Vec<u8>,
    _class_file: Option<ClassFile<'a>>,

    _mirror: ObjHandle,

    _fields: Fields<'a>,

    // hot fields
    _cp_entries: usize,
    _size_of_instance: usize
}

impl Klass<'static> {
    // Returning false means ClassNotFoundException.
    pub fn init_normal(
        &'static mut self,
        loader: ObjPtr,
        metadata: Vec<u8>,
    ) -> bool {
        *self = Self {
            _name: None,
            _super: None,
            _loader: ObjHandle::with_oop(loader),
            _metadata: Vec::new(),
            _class_file: None,
            _mirror: ObjHandle::new(),
            _fields: Fields::new(),
            _cp_entries: 0,
            _size_of_instance: 0
        };

        self._metadata = metadata;

        self._class_file = Some(match cafebabe::parse_class(self._metadata.as_slice()) {
            Ok(n) => n,
            Err(_) => return false,
        });
        let cf = self._class_file.as_ref().unwrap();

        self._name = Some(cf.this_class.clone());

        self._super = match cf.super_class.clone() {
            Some(s) => Some(class_loader::load_class(loader, s.to_string())),
            None => None
        };

        self._cp_entries = cf.constantpool_iter().count();

        // todo: resolve fields and methods.
        let offs = match self._super {
            Some(s) => s.size_of_instance(),
            None => ObjDesc::size_of_normal_desc()
        };
        self._fields.init(offs, &cf.fields);

        true
    }

    pub fn init_array_class(&mut self, elem_type: &'static Klass, dimensions: usize) {
        unimplemented!()
    }
}

impl Drop for Klass<'_> {
    fn drop(&mut self) {
        unreachable!()
    }
}

impl Klass<'_> {
    pub fn name(&self) -> ClassName {
        self._name.as_ref().unwrap().clone()
    }

    pub fn super_class(&self) -> Option<&'static Klass> {
        self._super
    }
    
    pub fn mirror(&self) -> &ObjHandle {
        &self._mirror
    }

    pub fn cp_entries(&self) -> usize {
        self._cp_entries
    }
}

impl Klass<'_> {
    pub fn size_of_instance(&self) -> usize {
        self._size_of_instance
    }
}
