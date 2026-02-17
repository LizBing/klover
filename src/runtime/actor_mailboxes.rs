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

use tokio::sync::mpsc;

use crate::{
    classfile::cld_actor::CLDMsg,
    gc::oop_storage_actor::OOPStorageMsg,
    metaspace::ms_actor::MSMsg
};

#[derive(Debug)]
pub struct ActorMailboxes {
    pub(super) cld_tx: mpsc::UnboundedSender<CLDMsg>,
    pub(super) oop_storage_tx: mpsc::UnboundedSender<OOPStorageMsg>,
    pub(super) ms_tx: mpsc::UnboundedSender<MSMsg>,
}

impl ActorMailboxes {
    pub fn send_cld(&self, msg: CLDMsg) {
        self.cld_tx.send(msg).unwrap()
    }

    pub fn send_oop_storage(&self, msg: OOPStorageMsg) {
        self.oop_storage_tx.send(msg).unwrap()
    }

    pub fn send_metaspace(&self, msg: MSMsg) {
        self.ms_tx.send(msg).unwrap()
    }
}
