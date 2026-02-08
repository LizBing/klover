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

use std::{ptr::NonNull, sync::{OnceLock, atomic::AtomicPtr}};

use tokio::sync::mpsc;

use crate::{classfile::{class_loader::ClassLoader, cld_actor::CLDMsg}, oops::{klass::Klass, oop_hierarchy::OOP}, runtime::universe::Universe};

pub struct JavaClasses {
    java_lang_Object: NonNull<Klass>,
    java_lang_String: NonNull<Klass>,
    java_lang_Class: NonNull<Klass>,
    java_lang_Throwable: NonNull<Klass>
}

/*
impl JavaClasses {
    pub async fn new() -> Self {
        let (tx, mut rx) = mpsc::channel(1);
        let msg = CLDMsg::GetCLD { loader: OOP::null(), reply_tx: tx };

        Universe::actor_mailbox().send_cld(msg);
        let cld = rx.recv().await.unwrap();

        Self {
            java_lang_Object: 
        }
    }
}
*/

impl JavaClasses {}
