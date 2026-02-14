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

use cafebabe::bytecode::Opcode;

use crate::oops::symbol::SymbolHandle;

pub struct RtObjectArrayType {}

pub struct RtNameAndType {
    name: SymbolHandle,
    desc: SymbolHandle
}

pub struct RtFieldRef {}

pub struct RtMethodRef {}

pub struct RtDynamicInfo {}

pub struct RtInvokeDynamicInfo {}

pub struct RtLoadable {}

pub struct RtLookupTable {}

pub struct RtRangeTable {}

pub enum RtCPItem {
    ObjectArrayType(RtObjectArrayType),
    FieldRef(RtFieldRef),
    MethodRef(RtMethodRef),
    DynamicInfo(RtDynamicInfo),
    InvokeDynamicInfo(RtInvokeDynamicInfo),
    Loadable(RtLoadable),
}
