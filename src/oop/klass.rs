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

use crate::oop::obj_handle::ObjHandle;

pub struct Klass {
    _super: *mut Klass,
    _mirror: ObjHandle,
    _define_loader: ObjHandle,  // Nullable, which refers to Bootstrap Loader.
}

impl Klass {
    pub fn size_of_instance() -> usize {
        unimplemented!()
    }
}
