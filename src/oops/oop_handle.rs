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

use once_cell::sync::OnceCell;

use crate::{gc::{barrier_set::AccessBarrier, oop_storage::OOPStorage}, oops::{access::{AccessAPI, DECORATOR_NOT_IN_HEAP}, oop_hierarchy::OOP}};

#[derive(Debug)]
pub struct OOPHandleAccessAPI {
    oop_load: fn(*const OOP) -> OOP,
    oop_store: fn(*const OOP, n: OOP)
}

static ACC_API: OnceCell<OOPHandleAccessAPI> = OnceCell::new();

// dispatch
pub fn initialize<Barrier: AccessBarrier>() {
    let api = OOPHandleAccessAPI {
        oop_load: AccessAPI::<DECORATOR_NOT_IN_HEAP>::oop_load::<Barrier, _>,
        oop_store: AccessAPI::<DECORATOR_NOT_IN_HEAP>::oop_store::<Barrier, _>
    };

    ACC_API.set(api).unwrap()
}

#[derive(Debug)]
pub struct OOPHandle {
    _obj: *mut OOP
}

impl OOPHandle {
    pub fn new(storage: &OOPStorage) -> Self {
        Self {
            _obj: storage.allocate()
        }
    }

    pub fn set(&self, n: OOP) {
        (ACC_API.get().unwrap().oop_store)(self._obj, n)
    }

    pub fn get(&self) -> OOP {
        (ACC_API.get().unwrap().oop_load)(self._obj)
    }
}
