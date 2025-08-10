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
use crate::oops::field::Field;
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

    _fields: Vec<Field<'a>>
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
            _fields: Vec::new(),
        };

        self._metadata = metadata;
        let md = &self._metadata;

        let cf = match cafebabe::parse_class(md.as_slice()) {
            Ok(n) => n,
            Err(_) => return false,
        };

        self._name = Some(cf.this_class.clone());

        self._super = match cf.super_class.clone() {
            Some(s) => Some(class_loader::load_class(loader, s.to_string())),
            None => None
        };

        self._class_file = Some(cf);

        true
    }

    pub fn init_array_class(&mut self, name: ClassName<'static>, loader: ObjPtr) {
        *self = Self {
            _name: Some(name),
            _super: Some(JavaLangObject::this()),
            _loader: ObjHandle::with_oop(loader),
            _metadata: Vec::new(),
            _class_file: None,
            _mirror: ObjHandle::new(),
            _fields: Vec::new(),
        }
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
}

impl Klass<'_> {
    pub fn size_of_instance() -> usize {
        unimplemented!()
    }
}
