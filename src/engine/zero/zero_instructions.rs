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

use std::{ffi::c_void, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub}, ptr::null};

use crate::{engine::{bytecodes, engine_runtime::{DStackSlot, StackSlot, StackSlotType}, zero::{zero_constrains::FloatType, zero_runtime::ZeroRegisters}}, oops::{access::{DECORATOR_IN_HEAP, DECORATOR_MO_VOLATILE}, oop_hierarchy::{ArrayOOP, NarrowOOP, OOP}}, utils::global_defs::{Address, JByte, JChar, JDouble, JFloat, JInt, JLong, JShort}};

pub struct ZeroInstructions;

macro_rules! type_op {
    ($name:ident, $trait_: ty, $op:tt) => {
        paste::paste! {
            fn [<type_ $name>]<T: Copy + $trait_>(regs: &mut ZeroRegisters) {
               let value2 = Self::pop::<T>(regs);
                let value1 = Self::pop::<T>(regs);

                let result = value1 $op value2;

                Self::push(regs, result); 
            }
        }
    };
}

pub type InsFnType = fn(regs: &mut ZeroRegisters<'_>);

pub static INS_TABLE: [InsFnType; bytecodes::NUMBER_OF_JAVA_CODES] = [
    ZeroInstructions::nop,

    ZeroInstructions::aconst_null,

    ZeroInstructions::int_type_const_n::<JInt, -1>, // iconst_m1
    ZeroInstructions::int_type_const_n::<JInt, 0>,  // iconst_0
    ZeroInstructions::int_type_const_n::<JInt, 1>,
    ZeroInstructions::int_type_const_n::<JInt, 2>,
    ZeroInstructions::int_type_const_n::<JInt, 3>,
    ZeroInstructions::int_type_const_n::<JInt, 4>,
    ZeroInstructions::int_type_const_n::<JInt, 5>,

    ZeroInstructions::int_type_const_n::<JLong, 0>,
    ZeroInstructions::int_type_const_n::<JLong, 1>,

    ZeroInstructions::float_type_const_0::<JFloat>,
    ZeroInstructions::float_type_const_1::<JFloat>,
    ZeroInstructions::float_type_const_2::<JFloat>,

    ZeroInstructions::float_type_const_0::<JDouble>,
    ZeroInstructions::float_type_const_1::<JDouble>,

    ZeroInstructions::bipush,
    ZeroInstructions::sipush,

    ZeroInstructions::ldc,
    ZeroInstructions::ldc_w,
    ZeroInstructions::ldc2_w,

    ZeroInstructions::type_load::<JInt>,
    ZeroInstructions::type_load::<JLong>,
    ZeroInstructions::type_load::<JFloat>,
    ZeroInstructions::type_load::<JDouble>,
    ZeroInstructions::aload,

    ZeroInstructions::type_load_n::<JInt, 0>,
    ZeroInstructions::type_load_n::<JInt, 1>,
    ZeroInstructions::type_load_n::<JInt, 2>,
    ZeroInstructions::type_load_n::<JInt, 3>,

    ZeroInstructions::type_load_n::<JLong, 0>,
    ZeroInstructions::type_load_n::<JLong, 1>,
    ZeroInstructions::type_load_n::<JLong, 2>,
    ZeroInstructions::type_load_n::<JLong, 3>,

    ZeroInstructions::type_load_n::<JFloat, 0>,
    ZeroInstructions::type_load_n::<JFloat, 1>,
    ZeroInstructions::type_load_n::<JFloat, 2>,
    ZeroInstructions::type_load_n::<JFloat, 3>,

    ZeroInstructions::type_load_n::<JDouble, 0>,
    ZeroInstructions::type_load_n::<JDouble, 1>,
    ZeroInstructions::type_load_n::<JDouble, 2>,
    ZeroInstructions::type_load_n::<JDouble, 3>,

    ZeroInstructions::aload_n::<0>,
    ZeroInstructions::aload_n::<1>,
    ZeroInstructions::aload_n::<2>,
    ZeroInstructions::aload_n::<3>,

    ZeroInstructions::type_aload::<JInt>,
    ZeroInstructions::type_aload::<JLong>,
    ZeroInstructions::type_aload::<JFloat>,
    ZeroInstructions::type_aload::<JDouble>,
    ZeroInstructions::aaload,
    ZeroInstructions::type_aload::<JByte>,
    ZeroInstructions::type_aload::<JChar>,
    ZeroInstructions::type_aload::<JShort>,

    ZeroInstructions::type_store::<JInt>,
    ZeroInstructions::type_store::<JLong>,
    ZeroInstructions::type_store::<JFloat>,
    ZeroInstructions::type_store::<JLong>,
    ZeroInstructions::astore,

    ZeroInstructions::type_store_n::<JInt, 0>,
    ZeroInstructions::type_store_n::<JInt, 1>,
    ZeroInstructions::type_store_n::<JInt, 2>,
    ZeroInstructions::type_store_n::<JInt, 3>,

    ZeroInstructions::type_store_n::<JLong, 0>,
    ZeroInstructions::type_store_n::<JLong, 1>,
    ZeroInstructions::type_store_n::<JLong, 2>,
    ZeroInstructions::type_store_n::<JLong, 3>,

    ZeroInstructions::type_store_n::<JFloat, 0>,
    ZeroInstructions::type_store_n::<JFloat, 1>,
    ZeroInstructions::type_store_n::<JFloat, 2>,
    ZeroInstructions::type_store_n::<JFloat, 3>,

    ZeroInstructions::type_store_n::<JDouble, 0>,
    ZeroInstructions::type_store_n::<JDouble, 1>,
    ZeroInstructions::type_store_n::<JDouble, 2>,
    ZeroInstructions::type_store_n::<JDouble, 3>,

    ZeroInstructions::astore_n::<0>,
    ZeroInstructions::astore_n::<1>,
    ZeroInstructions::astore_n::<2>,
    ZeroInstructions::astore_n::<3>,

    ZeroInstructions::type_astore::<JInt>,
    ZeroInstructions::type_astore::<JLong>,
    ZeroInstructions::type_astore::<JFloat>,
    ZeroInstructions::type_astore::<JDouble>,
    ZeroInstructions::aastore,
    ZeroInstructions::type_astore::<JByte>,
    ZeroInstructions::type_astore::<JChar>,
    ZeroInstructions::type_astore::<JShort>,

    ZeroInstructions::pop,
    ZeroInstructions::pop2,

    ZeroInstructions::dupn::<1>,
    ZeroInstructions::dupn_x1::<1>,
    ZeroInstructions::dupn_x2::<1>,
    ZeroInstructions::dupn::<2>,
    ZeroInstructions::dupn_x1::<2>,
    ZeroInstructions::dupn_x2::<2>,

    ZeroInstructions::swap,

    ZeroInstructions::type_add::<JInt>,
    ZeroInstructions::type_add::<JLong>,
    ZeroInstructions::type_add::<JFloat>,
    ZeroInstructions::type_add::<JDouble>,

    ZeroInstructions::type_sub::<JInt>,
    ZeroInstructions::type_sub::<JLong>,
    ZeroInstructions::type_sub::<JFloat>,
    ZeroInstructions::type_sub::<JDouble>,

    ZeroInstructions::type_mul::<JInt>,
    ZeroInstructions::type_mul::<JLong>,
    ZeroInstructions::type_mul::<JFloat>,
    ZeroInstructions::type_mul::<JDouble>,

    ZeroInstructions::type_div::<JInt>,
    ZeroInstructions::type_div::<JLong>,
    ZeroInstructions::type_div::<JFloat>,
    ZeroInstructions::type_div::<JDouble>,

    ZeroInstructions::type_rem::<JInt>,
    ZeroInstructions::type_rem::<JLong>,
    ZeroInstructions::type_rem::<JFloat>,
    ZeroInstructions::type_rem::<JDouble>,

    ZeroInstructions::int_type_neg::<JInt>,
    ZeroInstructions::int_type_neg::<JLong>,
    ZeroInstructions::float_type_neg::<JFloat>,
    ZeroInstructions::float_type_neg::<JDouble>,

    ZeroInstructions::type_shl::<JInt>,
    ZeroInstructions::type_shl::<JLong>,

    ZeroInstructions::type_shr::<JInt>,
    ZeroInstructions::type_shr::<JLong>,

    ZeroInstructions::type_shr::<u32>,
    ZeroInstructions::type_shr::<u64>,

    ZeroInstructions::type_and::<JInt>,
    ZeroInstructions::type_and::<JLong>,

    ZeroInstructions::type_or::<JInt>,
    ZeroInstructions::type_or::<JLong>,

    ZeroInstructions::type_xor::<JInt>,
    ZeroInstructions::type_xor::<JLong>,

    ZeroInstructions::iinc,

    ZeroInstructions::t1_to_t2::<JInt, JLong>,
    ZeroInstructions::t1_to_t2::<JInt, JFloat>,
    ZeroInstructions::t1_to_t2::<JInt, JDouble>,

    ZeroInstructions::t1_to_t2::<JLong, JInt>,
    ZeroInstructions::t1_to_t2::<JLong, JFloat>,
    ZeroInstructions::t1_to_t2::<JLong, JDouble>,

    ZeroInstructions::t1_to_t2::<JFloat, JInt>,
    ZeroInstructions::t1_to_t2::<JFloat, JLong>,
    ZeroInstructions::t1_to_t2::<JFloat, JDouble>,

    ZeroInstructions::t1_to_t2::<JDouble, JInt>,
    ZeroInstructions::t1_to_t2::<JDouble, JLong>,
    ZeroInstructions::t1_to_t2::<JDouble, JFloat>,

    ZeroInstructions::t1_to_t2::<JInt, JByte>,
    ZeroInstructions::t1_to_t2::<JInt, JChar>,
    ZeroInstructions::t1_to_t2::<JInt, JShort>,

    ZeroInstructions::lcmp,
    ZeroInstructions::type_cmpl::<JFloat>,
    ZeroInstructions::type_cmpg::<JFloat>,
    ZeroInstructions::type_cmpl::<JDouble>,
    ZeroInstructions::type_cmpg::<JDouble>,

    ZeroInstructions::ifeq,
    ZeroInstructions::ifne,
    ZeroInstructions::iflt,
    ZeroInstructions::ifge,
    ZeroInstructions::ifgt,
    ZeroInstructions::ifle,

    ZeroInstructions::if_type_cmpeq::<JInt>,
    ZeroInstructions::if_type_cmpne::<JInt>,
    ZeroInstructions::if_type_cmplt::<JInt>,
    ZeroInstructions::if_type_cmpge::<JInt>,
    ZeroInstructions::if_type_cmpgt::<JInt>,
    ZeroInstructions::if_type_cmple::<JInt>,

    ZeroInstructions::if_type_cmpeq::<JInt>,
    ZeroInstructions::if_type_cmpne::<JInt>,

    ZeroInstructions::goto,
    ZeroInstructions::jsr,
    ZeroInstructions::ret,
    ZeroInstructions::tableswitch,
    ZeroInstructions::lookupswitch,

    ZeroInstructions::type_return::<JInt>,
    ZeroInstructions::type_return::<JLong>,
    ZeroInstructions::type_return::<JFloat>,
    ZeroInstructions::type_return::<JDouble>,
    ZeroInstructions::areturn,
    ZeroInstructions::return_,

    ZeroInstructions::getstatic,
    ZeroInstructions::putstatic,
    ZeroInstructions::getfield,
    ZeroInstructions::putfield,

    ZeroInstructions::invokevirtual,
    ZeroInstructions::invokespecial,
    ZeroInstructions::invokestatic,
    ZeroInstructions::invokeinterface,
    ZeroInstructions::invokedynamic,

    ZeroInstructions::new,
    ZeroInstructions::newarray,
    ZeroInstructions::anewarray,
    ZeroInstructions::arraylength,
    ZeroInstructions::athrow,

    ZeroInstructions::checkcast,
    ZeroInstructions::instanceof,

    ZeroInstructions::monitorenter,
    ZeroInstructions::monitorexit,

    ZeroInstructions::wide,
    
    ZeroInstructions::multianewarray,

    ZeroInstructions::ifnull,
    ZeroInstructions::ifnonnull,

    ZeroInstructions::goto_w,
    ZeroInstructions::jsr_w,

    // ZeroInstructions::breakpoint,
];

// helpers
impl ZeroInstructions {
    const fn slots_of<T>() -> usize {
        let size = size_of::<T>();
        if size <= size_of::<StackSlot>() { 1 }
        else if size == size_of::<DStackSlot>() { 2 }
        else { unreachable!() }
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
        NarrowOOP::decode(Self::pop(regs))
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
        Self::push(regs, NarrowOOP::encode(oop));
    }

    #[inline]
    fn local_load_oop(regs: &mut ZeroRegisters, index: impl Into<usize>) -> OOP {
        NarrowOOP::decode(Self::local_load(regs, index))
    }

    #[inline]
    fn local_store_oop(regs: &mut ZeroRegisters, index: impl Into<usize>, value: OOP) {
        Self::local_store(regs, index, NarrowOOP::encode(value));
    }

    #[inline]
    fn local_load<T: Copy>(regs: &mut ZeroRegisters, index: impl Into<usize>) -> T {
        unsafe {
            *((*regs.bp).local(index) as *const _)
        }
    }

    #[inline]
    fn local_store<T>(regs: &mut ZeroRegisters, index: impl Into<usize>, value: T) {
        unsafe {
            *((*regs.bp).local(index) as *mut _) = value
        }
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
        let byte1 = Self::read_u8(regs) as u16;
        let byte2 = Self::read_u8(regs) as u16;

        (byte1 << 8) | byte2
    }

    #[inline]
    fn read_u32(regs: &mut ZeroRegisters) -> u32 {
        let short1 = Self::read_u16(regs) as u32;
        let short2 = Self::read_u16(regs) as u32;

        (short1 << 16) | short2
    }
}

impl ZeroInstructions {
    fn aaload(regs: &mut ZeroRegisters) {
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length::<{DECORATOR_IN_HEAP}>(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        let value = ArrayOOP::get_oop::<{DECORATOR_IN_HEAP | DECORATOR_MO_VOLATILE}>(arrayref, index);
        Self::push_ref(regs, value);
    }

    fn aastore(regs: &mut ZeroRegisters) {
        let value = Self::pop_ref(regs);
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length::<{DECORATOR_IN_HEAP}>(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        if !value.is_null() {
            // todo: Check if assignment compatible.
            // If not compatible, throw ArrayStoreException
        }

        ArrayOOP::put_oop::<{DECORATOR_IN_HEAP | DECORATOR_MO_VOLATILE}>(arrayref, index, value);
    }

    fn aconst_null(regs: &mut ZeroRegisters) {
        Self::push(regs, NarrowOOP::null());
    }

    fn aload(regs: &mut ZeroRegisters) {
        let index = Self::read_u8(regs);
        let objectref = Self::local_load_oop(regs, index);

        Self::push_ref(regs, objectref);
    }

    fn wide_aload(regs: &mut ZeroRegisters) {
        let index = Self::read_u16(regs);
        let objectref = Self::local_load_oop(regs, index);

        Self::push_ref(regs, objectref);
    }

    fn aload_n<const N: u8>(regs: &mut ZeroRegisters) {
        let objectref = Self::local_load_oop(regs, N);

        Self::push_ref(regs, objectref);
    }

    fn anewarray(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn areturn(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn arraylength(regs: &mut ZeroRegisters) {
        let arrayref = Self::pop_ref(regs);
        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        let length = ArrayOOP::length::<{DECORATOR_IN_HEAP}>(arrayref);
        Self::push(regs, length);
    }

    fn astore(regs: &mut ZeroRegisters) {
        let index = Self::read_u8(regs);
        let objectref = Self::pop_ref(regs);
        Self::local_store_oop(regs, index, objectref);
    }

    fn wide_astore(regs: &mut ZeroRegisters) {
        let index = Self::read_u16(regs);
        let objectref = Self::pop_ref(regs);

        Self::local_store_oop(regs, index, objectref);
    }

    fn astore_n<const N: u8>(regs: &mut ZeroRegisters) {
        let objectref = Self::pop_ref(regs);
        Self::local_store_oop(regs, N, objectref);
    }

    fn athrow(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn type_aload<T: Copy + Into<U>, U>(regs: &mut ZeroRegisters) {
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length::<{DECORATOR_IN_HEAP}>(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        let value = ArrayOOP::get::<{DECORATOR_IN_HEAP | DECORATOR_MO_VOLATILE}, T>(arrayref, index).into();
        Self::push(regs, value);
    }

    fn type_astore<T: Copy, U: Copy + Into<T>>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<U>(regs).into();
        let index = Self::pop(regs);
        let arrayref = Self::pop_ref(regs);

        if arrayref.is_null() {
            // todo: throw NullPointerException
        }

        if index >= ArrayOOP::length::<{DECORATOR_IN_HEAP}>(arrayref) {
            // todo: throw ArrayIndexOutOfBoundsException
        }

        ArrayOOP::put::<{DECORATOR_IN_HEAP | DECORATOR_MO_VOLATILE}, T>(arrayref, index, value);
    }

    fn bipush(regs: &mut ZeroRegisters) {
        let byte = Self::read_u8(regs);
        let value = byte.cast_signed() as i32;
        
        Self::push(regs, value);
    }

    fn checkcast(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn t1_to_t2<T1: Copy + Into<T2>, T2: Copy>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T1>(regs);
        let result = value.into();

        Self::push(regs, result);
    }

    type_op!(add, Add, +);

    fn type_cmpg<T: FloatType>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        // in case NaN
        let result = if value1 < value2 {
            -1
        } else if value1 == value2 {
            0
        } else {
            1
        };

        Self::push(regs, result);
    }

    fn type_cmpl<T: FloatType>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        // in case NaN
        let result = if value1 > value2 {
            1
        } else if value1 == value2 {
            0
        } else {
            -1
        };

        Self::push(regs, result);
    }

    fn float_type_const_0<T: FloatType>(regs: &mut ZeroRegisters) {
        Self::push(regs, T::ZERO);
    }

    fn float_type_const_1<T: FloatType>(regs: &mut ZeroRegisters) {
        Self::push(regs, T::ONE);
    }

    fn float_type_const_2<T: FloatType>(regs: &mut ZeroRegisters) {
        Self::push(regs, T::TWO);
    }

    fn type_div<T: Copy + Div>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        let result = value1 / value2;

        Self::push(regs, result);
    }

    fn type_load<T: Copy>(regs: &mut ZeroRegisters) {
        let index = Self::read_u8(regs);
        let value = Self::local_load::<T>(regs, index);

        Self::push(regs, value);
    }

    fn wide_type_load<T: Copy>(regs: &mut ZeroRegisters) {
        let index = Self::read_u16(regs);
        let value = Self::local_load::<T>(regs, index);

        Self::push(regs, value);
    }

    fn type_load_n<T: Copy, const N: usize>(regs: &mut ZeroRegisters) {
        let value = Self::local_load::<T>(regs, N);

        Self::push(regs, value);
    }

    type_op!(mul, Mul, *);

    fn float_type_neg<T: FloatType>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);

        let result = if value.is_nan() {
            T::NAN
        } else {
            -value
        };

        Self::push(regs, result);
    }

    type_op!(rem, Rem, %);

    fn type_return(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn type_store<T: Copy>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);
        let index = Self::read_u8(regs);

        Self::local_store(regs, index, value);
    }
    
    fn wide_type_store<T: Copy>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);
        let index = Self::read_u16(regs);

        Self::local_store(regs, index, value);
    }

    fn type_store_n<T: Copy, const N: usize>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);

        Self::local_store(regs, N, value);
    }

    type_op!(sub, Sub, -);

    fn dupn<T: StackSlotType>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);

        Self::push(regs, value);
        Self::push(regs, value);
    }

    fn dupn_x1<T: StackSlotType>(regs: &mut ZeroRegisters) {
        let value1 = Self::pop::<T>(regs);
        let value2 = Self::pop::<T>(regs);

        Self::push(regs, value1);
        Self::push(regs, value2);
        Self::push(regs, value1);
    }

    fn dupn_x2<T: StackSlotType>(regs: &mut ZeroRegisters) {
        let value1 = Self::pop::<T>(regs);
        let value2 = Self::pop::<T>(regs);
        let value3 = Self::pop::<T>(regs);

        Self::push(regs, value1);
        Self::push(regs, value3);
        Self::push(regs, value2);
        Self::push(regs, value1);
    }

    fn getfield(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn getstatic(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn goto(regs: &mut ZeroRegisters) {
        let branchoffset = Self::read_u16(regs).cast_signed() as isize;
        unsafe { regs.pc = regs.pc.byte_offset(branchoffset - 1) };
    }

    fn goto_w(regs: &mut ZeroRegisters) {
        let branchoffset = Self::read_u32(regs).cast_signed() as isize;
        unsafe { regs.pc = regs.pc.byte_offset(branchoffset - 1) };
    }

    type_op!(and, BitAnd, &);

    fn int_type_const_n<T: Copy + From<i8>, const N: i8>(regs: &mut ZeroRegisters) {
        Self::push(regs, T::from(N));
    }

    // For OOP, make T JInt
    fn if_type_cmpeq<T: Copy + PartialEq>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 == value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    // For OOP, make T JInt
    fn if_type_cmpne<T: Copy + PartialEq>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 != value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn if_type_cmplt<T: Copy + PartialOrd>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 < value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn if_type_cmple<T: Copy + PartialOrd>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 <= value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn if_type_cmpgt<T: Copy + PartialOrd>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 > value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }
    
    fn if_type_cmpge<T: Copy + PartialOrd>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<T>(regs);
        let value1 = Self::pop::<T>(regs);

        if value1 >= value2 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifeq(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value == 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifne(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value != 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn iflt(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value < 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifle(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value <= 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifgt(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value > 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifge(regs: &mut ZeroRegisters) {
        let value = Self::pop::<i32>(regs);

        if value >= 0 {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifnonnull(regs: &mut ZeroRegisters) {
        let value = Self::pop_ref(regs);

        if !value.is_null() {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn ifnull(regs: &mut ZeroRegisters) {
        let value = Self::pop_ref(regs);

        if value.is_null() {
            Self::goto(regs);
        } else {
            Self::read_u16(regs);
        }
    }

    fn iinc(regs: &mut ZeroRegisters) {
        let index = Self::read_u8(regs);
        let const_ = Self::read_u8(regs).cast_signed() as i32;

        let local = Self::local_load::<i32>(regs, index);

        let result = local + const_;

        Self::local_store(regs, index, result);
    }

    fn wide_iinc(regs: &mut ZeroRegisters) {
        let index = Self::read_u16(regs);
        let const_ = Self::read_u16(regs).cast_signed() as i32;

        let local = Self::local_load::<i32>(regs, index);

        let result = local + const_;

        Self::local_store(regs, index, result);
    }

    fn int_type_neg<T: Copy + Neg>(regs: &mut ZeroRegisters) {
        let value = Self::pop::<T>(regs);
        
        let result = -value;

        Self::push(regs, result);
    }

    fn instanceof(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn invokedynamic(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn invokeinterface(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn invokespecial(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn invokestatic(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn invokevirtual(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    type_op!(or, BitOr, |);

    fn type_shl<T: Copy + Shl<i32>>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<i32>(regs);
        let value1 = Self::pop::<T>(regs);

        let result = value1 << value2;

        Self::push(regs, result);
    }

    fn type_shr<T: Copy + Shr<i32>>(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<i32>(regs);
        let value1 = Self::pop::<T>(regs);

        let result = value1 >> value2;

        Self::push(regs, result);
    }

    type_op!(xor, BitXor, ^);

    fn jsr(regs: &mut ZeroRegisters) {
        panic!("Deprecated");
    }

    fn jsr_w(regs: &mut ZeroRegisters) {
        panic!("Deprecated");
    }

    fn lcmp(regs: &mut ZeroRegisters) {
        let value2 = Self::pop::<i64>(regs);
        let value1 = Self::pop::<i64>(regs);

        let result = if value1 > value2 {
            1i32
        } else if value1 == value2 {
            0i32
        } else if value1 < value2 {
            -1i32
        } else {
            unreachable!()
        };

        Self::push(regs, result);
    }

    fn ldc(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn ldc_w(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn ldc2_w(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn lookupswitch(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn monitorenter(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn monitorexit(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn multianewarray(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn new(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn newarray(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn nop(_regs: &mut ZeroRegisters) {
        // do nothing.
    }

    fn pop_(regs: &mut ZeroRegisters) {
        Self::pop::<StackSlot>(regs);
    }

    fn pop2(regs: &mut ZeroRegisters) {
        Self::pop::<DStackSlot>(regs);
    }

    fn putfield(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn putstatic(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn ret(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn wide_ret(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn return_(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn sipush(regs: &mut ZeroRegisters) {
        let value = Self::read_u16(regs).cast_signed() as i32;

        Self::push(regs, value);
    }

    fn swap(regs: &mut ZeroRegisters) {
        let value1 = Self::pop::<StackSlot>(regs);
        let value2 = Self::pop::<StackSlot>(regs);

        Self::push(regs, value1);
        Self::push(regs, value2);
    }

    fn tableswitch(regs: &mut ZeroRegisters) {
        unimplemented!()
    }

    fn wide(regs: &mut ZeroRegisters) {
        let opcode = Self::read_u8(regs);

        match opcode {
            bytecodes::_ILOAD => Self::wide_type_load::<i32>(regs),
            bytecodes::_FLOAD => Self::wide_type_load::<f32>(regs),
            bytecodes::_ALOAD => Self::wide_aload(regs),
            bytecodes::_LLOAD => Self::wide_type_load::<i64>(regs),
            bytecodes::_DLOAD => Self::wide_type_load::<f64>(regs),

            bytecodes::_ISTORE => Self::wide_type_store::<i32>(regs),
            bytecodes::_FSTORE => Self::wide_type_store::<f32>(regs),
            bytecodes::_ASTORE => Self::wide_astore(regs),
            bytecodes::_LSTORE => Self::wide_type_store::<i64>(regs),
            bytecodes::_DSTORE => Self::wide_type_store::<f64>(regs),

            bytecodes::_RET => Self::wide_ret(regs),

            bytecodes::_IINC => Self::wide_iinc(regs),
            
            _ => panic!("bad op code")
        }
    }
}
