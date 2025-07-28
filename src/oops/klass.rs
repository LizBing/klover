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
use crate::class_data::class_loader::ClassLoader;
use crate::oop::obj_handle::ObjHandle;
use crate::oop::oop::ObjPtr;

pub struct Klass {
    _name: &'_ ClassName<'_>,
    _super: *mut Klass,
    _loader: Option<Arc<ClassLoader>>,

    _class_file: Option<ClassFile<'_>>,
    _metadata: Vec<u8>,

    _mirror: ObjHandle,
}

impl Klass {
    pub fn init(&mut self, name: &ClassName, metadata: Vec<u8>) -> bool {
        self._name = name;
        self._super = null_mut();
        self._mirror = ObjHandle::new();

        if !metadata.is_empty() {
            match cafebabe::parse_class(metadata.as_slice()) {
                Ok(cf) => self._class_file = Some(cf),
                Err(e) => return false,
            }
        } else {
            self._class_file = None;
        }

        self._metadata = metadata;

        true
    }
}

impl Drop for Klass {
    fn drop(&mut self) {
        unreachable!()
    }
}

impl Klass {
    pub fn name(&self) -> &ClassName {
        self._name
    }

    pub fn super_class(&self) -> *mut Klass {
        unimplemented!()
    }
    
    pub fn mirror(&self) -> ObjPtr {
        self._mirror.oop()
    }
    
    pub fn set_mirror(&mut self, oop: ObjPtr) {
        self._mirror.set_oop(oop)
    }
}

impl Klass {
    pub fn size_of_instance() -> usize {
        unimplemented!()
    }
}
