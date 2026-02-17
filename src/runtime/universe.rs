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

use tokio::sync::mpsc;

use crate::{
    classfile::{
        cld_actor::CLDActor,
        symbol_table::SymbolTable
    },
    gc::{
        managed_heap::ManagedHeap,
        oop_storage_actor::OOPStorageActor
    },
    memory::compressed_space::NarrowEncoder,
    metaspace::ms_actor::MSActor,
    runtime::{
        actor_mailboxes::ActorMailboxes,
        vm_flags::VMFlags
    }
};

static UNIVERSE: OnceLock<Universe> = OnceLock::new();

#[derive(Debug)]
pub struct Universe {
    symbol_table: SymbolTable,
    heap: ManagedHeap,
    ms_narrow_encoder: NarrowEncoder,
    actor_mailboxes: ActorMailboxes,
    vm_flags: VMFlags
}

impl Universe {
    pub fn initialize() {
        let vm_flags = VMFlags::new();
        // vm_flags.init();

        let symbol_table = SymbolTable::new();

        let heap = ManagedHeap::new(vm_flags.xmx.clone());
        // heap.init();

        let (cld_tx, cld_rx) = mpsc::unbounded_channel();
        let (oop_storage_tx, oop_storage_rx) = mpsc::unbounded_channel();
        let (ms_tx, ms_rx) = mpsc::unbounded_channel();

        let mailboxes = ActorMailboxes {
            cld_tx: cld_tx,
            oop_storage_tx: oop_storage_tx,
            ms_tx: ms_tx
        };

        let cld_actor = CLDActor::new(cld_rx);

        let oop_storage_actor = OOPStorageActor::new(oop_storage_rx);

        let ms_actor = MSActor::new(ms_rx, *vm_flags.xmx);
        let mne = ms_actor.create_narrow_encoder();

        UNIVERSE.set(Self {
            symbol_table: symbol_table,
            heap: heap,
            ms_narrow_encoder: mne,
            actor_mailboxes: mailboxes,
            vm_flags: vm_flags
        }).unwrap();

        tokio::spawn(cld_actor.run());
        tokio::spawn(oop_storage_actor.run());
        tokio::spawn(ms_actor.run());
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

    pub fn symbol_table() -> &'static SymbolTable {
        &Self::this().symbol_table
    }
}
