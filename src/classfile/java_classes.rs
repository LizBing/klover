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
    fn name() -> &'static str;

    fn klass() -> NonNull<Klass> {
        static CACHE: OnceLock<AtomicPtr<Klass>> = OnceLock::new();

        let klass = CACHE.get_or_init(|| -> _ {
            let klass = ClassLoader::load_class(None, String::from(Self::name())).unwrap();
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
                fn name() -> &'static str {
                    VMSymbols:: $symbol_name ()
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

macro_rules! define_java_prim_class {
    (
        $type_name:ty, $symbol_name:ident
    ) => {
        paste::paste! {
            #[repr(C)]
            pub struct $type_name;

            impl JavaClass for $type_name {
                fn name() -> &'static str {
                    VMSymbols:: $symbol_name ()
                }
            }
        }
    }
}

define_java_prim_class!(PrimBool, prim_bool);
define_java_prim_class!(PrimByte, prim_byte);
define_java_prim_class!(PrimChar, prim_char);
define_java_prim_class!(PrimShort, prim_short);
define_java_prim_class!(PrimInt, prim_int);
define_java_prim_class!(PrimLong, prim_long);
define_java_prim_class!(PrimFloat, prim_float);
define_java_prim_class!(PrimDouble, prim_double);

define_java_class! {
    [JavaLangClassLoader, java_lang_ClassLoader] {}
}

define_java_class! {
    [JavaLangThrowable, java_lang_Throwable] {}
}
