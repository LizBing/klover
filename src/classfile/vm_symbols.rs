/*
 * Copyright 2026 Lei Zaakjyu
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

// @cl: cl!(ident, expr)
// e.g. $cl!(java_lang_Object, "java/lang/Object")
macro_rules! define_vm_symbols {
    (
        $(($symbol_name:ident, $str_form:expr))*
    ) => {
        impl VMSymbols {
            $(pub fn $symbol_name() -> &'static str { $str_form })*
        }
    }
}

pub struct VMSymbols;
define_vm_symbols! {
    (prim_bool, "bool")
    (prim_byte, "byte")
    (prim_char, "char")
    (prim_short, "short")
    (prim_int, "int")
    (prim_long, "long")
    (prim_float, "float")
    (prim_double, "double")

    (java_lang_System, "java/lang/System")
    (java_lang_Object, "java/lang/Object")
    (java_lang_Class, "java/lang/Class")
    (java_lang_ClassLoader, "java/lang/ClassLoader")
    (java_lang_Throwable, "java/lang/Throwable") 
}
