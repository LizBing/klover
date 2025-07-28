use std::cell::RefMut;
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
use std::ptr::null_mut;
use std::sync::Arc;
use cafebabe::ClassFile;
use cafebabe::descriptors::ClassName;
use crate::class_data::bootstrap_loader;
use crate::class_data::class_loader::ClassLoader;
use crate::class_data::java_classes::{JavaLangClass, JavaLangObject};
use crate::metaspace::klass_cell::KlassCell;
use crate::oops::obj_handle::ObjHandle;
use crate::oops::oop::ObjPtr;

pub struct Klass<'a> {
    _name: ClassName<'a>,
    _super: Option<KlassCell>,
    _loader: Option<Arc<ClassLoader>>,

    _metadata: Option<Vec<u8>>,
    _class_file: Option<ClassFile<'a>>,

    _mirror: ObjHandle,
}

impl<'a> Klass<'a> {
    // Returning false means ClassNotFoundException.
    pub fn init_normal<'b: 'a>(
        &'b mut self,
        loader: Option<Arc<ClassLoader>>,
        metadata: Vec<u8>,
    ) -> bool {
        self._metadata = Some(metadata);
        let md = self._metadata.as_ref().unwrap();

        let cf = match cafebabe::parse_class(md.as_slice()) {
            Ok(n) => n,
            Err(_) => return false,
        };

        self._name = cf.this_class.clone();
        self._super = match cf.super_class.clone() {
            Some(s) => match loader.as_ref() {
                Some(l) => l.load_class(s),
                None => bootstrap_loader::load_class(s)
            }

            None => None
        };
        self._loader = loader;
        self._class_file = Some(cf);

        // resolve the handle in define_class
        self._mirror = ObjHandle::new();

        true
    }

    pub fn init_array_class(&mut self, name: ClassName<'a>, loader: Option<Arc<ClassLoader>>) {
        self._name = name;
        self._super = Some(JavaLangObject::this());
        self._loader = loader;
        
        self._metadata = None;
        self._class_file = None;
        
        self._mirror = ObjHandle::new();
    }
}

impl Drop for Klass<'_> {
    fn drop(&mut self) {
        unreachable!()
    }
}

impl Klass<'_> {
    pub fn name(&self) -> ClassName {
        self._name.clone()
    }

    pub fn super_class(&self) -> Option<KlassCell> {
        unimplemented!()
    }
    
    pub fn mirror(&self) -> ObjPtr {
        self._mirror.oop()
    }
    
    pub fn set_mirror(&mut self, oop: ObjPtr) {
        self._mirror.set_oop(oop)
    }
}

impl Klass<'_> {
    pub fn size_of_instance() -> usize {
        unimplemented!()
    }
}
