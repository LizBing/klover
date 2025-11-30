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

use std::{marker::PhantomData, sync::Barrier};

use crate::{engine::{engine_runtime::{SLOTS_PER_INT, SLOTS_PER_REF, StackSlot}, zero::zero_runtime::ZeroRegisters}, gc::barrier_set::AccessBarrier, oops::{access::*, obj_desc::ArrayObjDesc, oop_hierarchy::{ArrayOOP, OOP}}};

#[inline]
fn pop<T: Copy, const SLOTS: usize>(rsp: &mut *const StackSlot) -> T {
    let res = unsafe { *(*rsp as *const _) };
    *rsp = rsp.wrapping_add(SLOTS);

    res
}

#[inline]
fn pop_ref<Barrier: AccessBarrier>(rsp: &mut *const StackSlot) -> OOP {
    let res = AccessAPI::<DECORATOR_NOT_IN_HEAP>::oop_load::<Barrier, _>(*rsp);
    *rsp = rsp.wrapping_add(SLOTS_PER_REF);

    res
}

#[inline]
fn push<T, const SLOTS: usize>(rsp: &mut *const StackSlot, value: T) {
    *rsp = rsp.wrapping_sub(SLOTS);
    unsafe { *(*rsp as *mut T) = value };
}

#[inline]
fn push_ref<Barrier: AccessBarrier>(rsp: &mut *const StackSlot, n: OOP) {
    *rsp = rsp.wrapping_sub(SLOTS_PER_REF);
    AccessAPI::<DECORATOR_NOT_IN_HEAP>::oop_store::<Barrier, _>(*rsp, n);
}

struct ZeroTemplates;

impl ZeroTemplates {
    pub fn aaload<Barrier: AccessBarrier>(regs: &mut ZeroRegisters) {
        let index = pop::<u32, SLOTS_PER_INT>(&mut regs.sp) as usize;
        let arrayref = pop_ref::<Barrier>(&mut regs.sp);

        if arrayref.is_null() {
            // todo: throw NullPointerException.
        }

        if index >= ArrayOOP::length(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        let value = AccessAPI::<DECORATOR_IN_HEAP>::oop_load_at::<Barrier>(
            arrayref, ArrayObjDesc::data_start_offset() + size_of::<OOP>() * index);

        push_ref::<Barrier>(&mut regs.sp, value);
    }

    pub fn aastore<Barrier: AccessBarrier>(regs: &mut ZeroRegisters) {
        let value = pop_ref::<Barrier>(&mut regs.sp);
        let index = pop::<u32, SLOTS_PER_INT>(&mut regs.sp) as usize;
        let arrayref = pop_ref::<Barrier>(&mut regs.sp);

        if arrayref.is_null() {
            // todo: throw NullPointerException.
        }

        if index >= ArrayOOP::length(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        // todo: check if value is assignment compatible with
        // the array component type(See 6.5 aastore)

        AccessAPI::<DECORATOR_IN_HEAP>::oop_store_at::<Barrier>(
            arrayref, ArrayObjDesc::data_start_offset() + size_of::<OOP>() * index, value);
    }

    pub fn aconst_null(regs: &mut ZeroRegisters) {
        // I don't know if it is right to push a null value without barrier.
        push::<_, SLOTS_PER_REF>(&mut regs.sp, OOP::null());
    }

    pub fn aload<Barrier: AccessBarrier>(regs: &mut ZeroRegisters) {
        unsafe { regs.pc = regs.pc.byte_add(1); }
        let index = unsafe { *regs.pc } as usize;

        let objectref = unsafe { *((*regs.bp).locals.add(index) as *const OOP) };
        
        push_ref::<Barrier>(&mut regs.sp, objectref);
    }

    pub fn aload_n<Barrier: AccessBarrier, const INDEX: usize>(regs: &mut ZeroRegisters) {
        let objectref = unsafe { *((*regs.bp).locals.add(INDEX) as *const OOP) };
        
        push_ref::<Barrier>(&mut regs.sp, objectref);
    }
}
