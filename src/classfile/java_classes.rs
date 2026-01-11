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

use std::{cell::OnceCell, mem::offset_of, ptr::NonNull, sync::{Arc, LazyLock, OnceLock, Weak, atomic::AtomicPtr}};

use crate::{classfile::{class_loader::ClassLoader, class_loader_data::ClassLoaderData}, oops::{klass::Klass, obj_desc::ObjDesc, oop_hierarchy::OOP}, utils::global_defs::JInt};
use super::vm_symbols::VMSymbols;

pub trait JavaClass {
    fn name() -> String;

    fn klass() -> NonNull<Klass> {
        static CACHE: OnceLock<AtomicPtr<Klass>> = OnceLock::new();

        let klass = CACHE.get_or_init(|| -> _ {
            let klass = ClassLoader::load_normal_class(None, Self::name()).unwrap();
            AtomicPtr::new(klass.as_ptr())
        });

        unsafe {
            NonNull::new_unchecked(*klass.as_ptr())
        }
    }
}

macro_rules! define_java_class {
    (
        [$type_name:ty, $symbol_name:ident] {
            $($field_name:ident : $field_type:ty,)*
        }
    ) => {
        paste::paste! {
            #[repr(C)]
            pub struct $type_name {
                _desc: ObjDesc,

                $(
                    $field_name: $field_type,
                )*
            }

            impl JavaClass for $type_name {
                fn name() -> String {
                    String::from(VMSymbols:: $symbol_name ())
                }
            }

            impl $type_name {
                $(
                    pub const fn [<$field_name _offset>]() -> usize {
                        offset_of!(Self, $field_name)
                    }
                )*
            }
        }
    }
}

define_java_class! {
    [JavaLangClassLoader, java_lang_ClassLoader] {
        cld: *const ClassLoaderData,
    }
}

define_java_class! {
    [JavaLangThrowable, java_lang_Throwable] {}
}
