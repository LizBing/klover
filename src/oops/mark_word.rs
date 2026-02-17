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

use modular_bitfield::{
    Specifier,
    bitfield,
    prelude::{
        B4,
        B26,
        B32,
        B62
    }
};

#[cfg(target_endian = "big")]
compile_error!("Klover VM currently only supports little endian.");

#[derive(Specifier, Clone, Copy)]
#[bits = 2]
pub enum LockType {
    NoLock,
    LWLocked,
    HWLocked,
    GCLocked
}

#[derive(Clone, Copy)]
#[bitfield(bits = 64)]
pub struct LockTypeIndicator {
    pub lock_type: LockType,
    #[skip] __: B62
}

#[bitfield(bits = 64)]
#[derive(Specifier, Clone, Copy)]
pub struct NoLockMarkWord {
    lock_type: LockType,

    pub age: B4,
    pub hash: B26,
    pub comp_klass_ptr: B32,
}

#[bitfield(bits = 64)]
#[derive(Specifier, Clone, Copy)]
pub struct LockedMarkWord {
    lock_type: LockType,

    pub lock: B62
}

pub union MarkWord {
    pub indicator: LockTypeIndicator,
    pub no_lock: NoLockMarkWord,
    pub locked: LockedMarkWord,

    pub raw: u64,
}

impl MarkWord {
    pub fn prototype() -> MarkWord {
        MarkWord {
            no_lock: NoLockMarkWord::new()
                .with_lock_type(LockType::NoLock)
                .with_age(0)
                .with_hash(0)
                .with_comp_klass_ptr(0)
        }
    }
}
