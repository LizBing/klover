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

use std::sync::OnceLock;

use crate::{gc::managed_heap::ManagedHeap, memory::compressed_space::NarrowEncoder, runtime::{actor_mailboxes::ActorMailboxes, vm_flags::VMFlags}};

static UNIVERSE: OnceLock<Universe> = OnceLock::new();

#[derive(Debug)]
pub struct Universe {
    heap: ManagedHeap,
    ms_narrow_encoder: NarrowEncoder,
    actor_mailboxes: ActorMailboxes,
    vm_flags: VMFlags
}

impl Universe {
    pub fn initialize() {
        unimplemented!()
    }
}

impl Universe {
    fn this() -> &'static Universe {
        UNIVERSE.get().expect("Should call Universe::initialize() in advance.")
    }

    pub fn heap() -> &'static ManagedHeap {
        &Self::this().heap
    }

    pub fn actor_mailboxes() -> &'static ActorMailboxes {
        &Self::this().actor_mailboxes
    }

    pub fn vm_flags() -> &'static VMFlags {
        &Self::this().vm_flags
    }

    pub fn ms_narrow_encoder() -> &'static NarrowEncoder {
        &Self::this().ms_narrow_encoder
    }
}
