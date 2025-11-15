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

use std::marker::PhantomData;

use bitflags::bitflags;

use crate::gc::barrier_set::AccessBarrier;

bitflags! {
    pub struct DecoratorSet : u32 {
        const IN_HEAP = 1u32 << 0;
        const NOT_IN_HEAP = 1u32 << 1;
    }
}

pub struct AccessAPI<Barrier: AccessBarrier<D>, const D: u32>(PhantomData<Barrier>);

impl<Barrier: AccessBarrier<D>, const D: u32> AccessAPI<Barrier, D> {}
