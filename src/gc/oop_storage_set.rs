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

use std::{array::from_fn, sync::OnceLock};

use crate::gc::oop_storage::OOPStorage;

const STRONG_COUNT: usize = 1;
const WEAK_COUNT: usize = 1;
const ALL_COUNT: usize = STRONG_COUNT + WEAK_COUNT;

const ALL_START: usize = 0;
const ALL_END: usize = ALL_START + ALL_COUNT;
const STRONG_START: usize = ALL_START;
const STRONG_END: usize = STRONG_START + STRONG_COUNT;
const WEAK_START: usize = STRONG_END;
const WEAK_END: usize = WEAK_START + WEAK_COUNT;

const CLD_WEAK_STORAGE_INDEX: usize = WEAK_START + 0;

#[derive(Debug)]
pub struct OOPStorageSet {
    _array: [OOPStorage; ALL_COUNT]
}

unsafe impl Send for OOPStorageSet {}
unsafe impl Sync for OOPStorageSet {}

static SET: OnceLock<OOPStorageSet> = OnceLock::new();
impl OOPStorageSet {
    fn new() -> Self {
        Self {
            _array: from_fn(|_| -> _ {
                OOPStorage::new()
            })
        }
    }

    pub fn initialize() {
        SET.set(Self::new()).unwrap()
    }
}

impl OOPStorageSet {
    fn set() -> &'static OOPStorageSet {
        SET.get().expect("Should be initialized in advance.")
    }

    pub fn storage(index: usize) -> &'static OOPStorage {
        &Self::set()._array[index]
    }

    pub fn strong_storage(index: usize) -> &'static OOPStorage {
        let index_ = index + STRONG_START;
        assert!(STRONG_START <= index_ && index_ < STRONG_END);

        Self::storage(index_)
    }

    pub fn weak_storage(index: usize) -> &'static OOPStorage {
        let index_ = index + WEAK_START;
        assert!(WEAK_START <= index_ && index_ < WEAK_START);

        Self::storage(index_)
    }
}

impl OOPStorageSet {
    pub fn cld_weak_storage() -> &'static OOPStorage {
        Self::weak_storage(CLD_WEAK_STORAGE_INDEX)
    }
}
