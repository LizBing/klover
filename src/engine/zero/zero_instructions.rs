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

use cafebabe::bytecode::Opcode;

use crate::{engine::{engine_runtime::StackSlot, zero::zero_runtime::ZeroRegisters}, oops::{access::{Access, DECORATOR_MO_VOLATILE, DECORATOR_NONE}, oop_hierarchy::{ArrayOOP, NarrowOOP, OOP}}};

// D: DECORATOR_INTERNAL_COMPRESSED
pub struct ZeroInstructions;

// helpers
impl ZeroInstructions {
    const fn slots_of<T>() -> usize {
        let size = size_of::<T>();
        if size <= size_of::<StackSlot>() { 1 }
        else { size / size_of::<StackSlot>() }
    }

    #[inline]
    fn pop<T: Copy>(regs: &mut ZeroRegisters) -> T {
        unsafe {
            let res = *(regs.sp as *const _);
            regs.sp = regs.sp.add(Self::slots_of::<T>());

            res
        }
    }

    #[inline]
    fn pop_ref(regs: &mut ZeroRegisters) -> OOP {
        let res = Access::<{DECORATOR_NONE}>::oop_load(regs.sp);
        regs.sp = unsafe { regs.sp.add(Self::slots_of::<NarrowOOP>()) };

        res
    }

    #[inline]
    fn push<T>(regs: &mut ZeroRegisters, value: T) {
        unsafe {
            regs.sp = regs.sp.sub(Self::slots_of::<T>());
            *(regs.sp as *mut _) = value;
        }
    }

    #[inline]
    fn push_ref(regs: &mut ZeroRegisters, oop: OOP) {
        regs.sp = unsafe { regs.sp.sub(Self::slots_of::<NarrowOOP>()) };
        Access::<{DECORATOR_NONE}>::oop_store(regs.sp, oop);
    }

    #[inline]
    fn load_local_oop(regs: &mut ZeroRegisters, index: u8) -> OOP {
        let addr = unsafe { (*regs.bp).local(index) };
        Access::<{DECORATOR_NONE}>::oop_load(addr)
    }

    #[inline]
    fn read_u8(regs: &mut ZeroRegisters) -> u8 {
        unsafe {
            regs.pc = regs.pc.add(1);
            *regs.pc
        }
    }

    #[inline]
    fn read_u16(regs: &mut ZeroRegisters) -> u16 {
        unsafe {
            regs.pc = regs.pc.add(1);
            let byte1 = *regs.pc as u16;
            regs.pc = regs.pc.add(1);
            let byte2 = *regs.pc as u16;

            (byte1 << 8) | byte2
        }
    }
}

impl ZeroInstructions {
    pub fn aaload(regs: &mut ZeroRegisters) {
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        let value = Access::<{DECORATOR_MO_VOLATILE}>::oop_load_at(arrayref, ArrayOOP::cal_offset::<NarrowOOP>(index));
        Self::push_ref(regs, value);
    }

    pub fn aastore(regs: &mut ZeroRegisters) {
        let value = Self::pop_ref(regs);
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        if !value.is_null() {
            // todo: Check if assignment compatible.
            // If not compatible, throw ArrayStoreException
        }

        Access::<{DECORATOR_MO_VOLATILE}>::oop_store_at(arrayref, ArrayOOP::cal_offset::<NarrowOOP>(index), value);
    }

    pub fn aconst_null(regs: &mut ZeroRegisters) {
        Self::push(regs, NarrowOOP::null());
    }

    pub fn aload(regs: &mut ZeroRegisters) {
        let index = Self::read_u8(regs);
        let objectref = Self::load_local_oop(regs, index);

        Self::push_ref(regs, objectref);
    }

    pub fn aload_n<const N: u8>(regs: &mut ZeroRegisters) {
        let objectref = Self::load_local_oop(regs, N);

        Self::push_ref(regs, objectref);
    }

    pub fn anewarray(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    pub fn areturn(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    pub fn arraylength(regs: &mut ZeroRegisters) {
    }
}
