/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::{cell::{Cell, UnsafeCell}, ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub}, result};

use cafebabe::bytecode::{ByteCode, Opcode};
use num::Float;

use crate::{ code::method::Method, jni::{jbyte, jchar, jdouble, jfloat, jint, jlong}, object::{klass::Klass, obj_desc::{ArrayObjDesc, ObjDesc}}, util::global_defs::{addr_cast, address}};

use super::{interpreter_runtime::{InterpreterRegisters, InterpreterStack}, slot_t};

const SLOT_PER_PTR: usize = 1;

const fn cal_slots<T>() -> usize {
    let size_of_T = size_of::<T>();
    let mut res = 0;

    if size_of_T <= 4 {
        res = 1;
    } else {
        res = 2;
    }

    debug_assert!(res == 1 || res == 2, "bad code");

    res
}

pub struct Executor<'a> {
    _regs: UnsafeCell<InterpreterRegisters<'a>>,
    _mthd: Cell<&'a Method<'a>>,
    _stack: InterpreterStack<'a>,
}

impl<'a> Executor<'a> {
    pub fn new(mthd: &'a Method<'a>) -> Self {
        Self {
            _regs: UnsafeCell::new(InterpreterRegisters::new()),
            _mthd: Cell::new(mthd),
            _stack: InterpreterStack::new(),
        }
    }

    pub fn init(&'a mut self, stack_size: usize) {
        self._stack.init(stack_size, &self._regs);
    }
}

impl<'a> Executor<'a> {
    fn regs(&self) -> &mut InterpreterRegisters<'a> {
        unsafe { &mut *(self._regs.get()) }
    }

    fn code(&self) -> &ByteCode {
        self._mthd.get().code_data().unwrap().bytecode.as_ref().unwrap()
    }
}

impl<'a> Executor<'a> {
    fn throw_exception(&mut self, e: &Klass) {
        unimplemented!()
    }

    fn pop_valid_obj<T>(&self) -> Option<&T> {
        unimplemented!()
    }

    fn array_load<T>(&self) -> Option<()> {
        let index = self._stack.pop(cal_slots::<jint>());
        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;

        let opt = arrayref.get::<T>(index);
        if let None = opt {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        let value = opt.unwrap();
        self._stack.push(cal_slots::<T>(), value);

        None
    }
    
    fn array_load_ptr(&self) -> Option<()> {
        let index = self._stack.pop(cal_slots::<jint>());
        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;

        let opt = arrayref.get_ptr(index);
        if let None = opt {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        let value = opt.unwrap();
        self._stack.push(SLOT_PER_PTR, value);

        None
    }
    
    fn array_store<T: Copy>(&self) -> Option<()> {
        let value = self._stack.pop::<T>(cal_slots::<T>());
        let index = self._stack.pop(cal_slots::<jint>());
        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;

        if !arrayref.put(index, value) {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        None
    }

    fn array_store_ptr(&self) -> Option<()> {
        let value = self._stack.pop(SLOT_PER_PTR);
        let index = self._stack.pop(cal_slots::<jint>());
        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;

        if !arrayref.put_ptr(index, value) {
            // todo: throw ArrayIndexOutOfBoundsException.
        }

        None
    }

    fn local_load<T: Copy>(&self, index: u16) {
        let value = self._stack.load_local::<T>(index);
        self._stack.push(cal_slots::<T>(), value);
    }

    fn local_load_ptr(&self, index: u16) {
        let objectref = self._stack.load_local::<address>(index);

        // todo: barrier, resolve oop map.

        self._stack.push(SLOT_PER_PTR, objectref);
    }

    fn local_store<T: Copy>(&self, index: u16) {
        let value = self._stack.pop::<T>(cal_slots::<T>());
        self._stack.store_local(index, value);
    }

    fn local_store_ptr(&self, index: u16) {
        let objectref = self._stack.pop::<address>(SLOT_PER_PTR);

        // todo: barrier, reslove oop map.

        self._stack.store_local(index, objectref);
    }

    // Returning false means that this stack is empty.
    fn return_with_value<T: Copy>(&self, slots: usize) -> bool {
        let value = self._stack.pop::<T>(slots);

        match self._stack.unwind() {
            Some(mthd) => {
                self._mthd.set(mthd);
                self._stack.push(slots, value);
            },

            _ => return false
        }

        true
    }
    
    fn return_void(&self) -> bool {
        match self._stack.unwind() {
            Some(mthd) => {
                self._mthd.set(mthd);
            },

            _ => return false
        }

        true
    }

    fn dup(&self) {
        let value = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value);
        self._stack.push(1, value);
    }

    fn dup_x1(&self) {
        let value1 = self._stack.pop::<slot_t>(1);
        let value2 = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value1);
        self._stack.push(1, value2);
        self._stack.push(1, value1);
    }

    fn dup_x2(&self) {
        let value1 = self._stack.pop::<slot_t>(1);
        let value2 = self._stack.pop::<slot_t>(1);
        let value3 = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value1);
        self._stack.push(1, value3);
        self._stack.push(1, value2);
        self._stack.push(1, value1);
    }

    fn dup2(&self) {
        let value1 = self._stack.pop::<slot_t>(1);
        let value2 = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value2);
        self._stack.push(1, value1);
        self._stack.push(1, value2);
        self._stack.push(1, value1);
    }

    fn dup2_x1(&self) {
        let value1 = self._stack.pop::<slot_t>(1);
        let value2 = self._stack.pop::<slot_t>(1);
        let value3 = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value2);
        self._stack.push(1, value1);
        self._stack.push(1, value3);
        self._stack.push(1, value2);
        self._stack.push(1, value1);
    }

    fn dup2_x2(&self) {
        let value1 = self._stack.pop::<slot_t>(1);
        let value2 = self._stack.pop::<slot_t>(1);
        let value3 = self._stack.pop::<slot_t>(1);
        let value4 = self._stack.pop::<slot_t>(1);

        self._stack.push(1, value2);
        self._stack.push(1, value1);
        self._stack.push(1, value4);
        self._stack.push(1, value3);
        self._stack.push(1, value2);
        self._stack.push(1, value1);
    }

    fn primitive_cast<FromType, ToType>(&self) { unimplemented!() }

    fn add<T>(&self)
    where T: Copy + Add {
        let slots = cal_slots::<T>();

        let value2 = self._stack.pop::<T>(slots);
        let value1 = self._stack.pop::<T>(slots);

        let result = value1 + value2;

        self._stack.push(slots, result);
    }

    fn sub<T>(&self)
    where  T: Copy + Sub { unimplemented!() }

    fn mul<T>(&self)
    where T: Copy + Mul { unimplemented!() }

    fn div<T>(&self)
    where T: Copy + Div { unimplemented!() }

    fn neg<T>(&self)
    where T: Copy + Neg { unimplemented!() }

    fn rem<T>(&self)
    where T: Copy + Rem { unimplemented!() }

    fn and<T>(&self)
    where T: Copy + BitAnd { unimplemented!() }

    fn or<T>(&self)
    where T: Copy + BitOr { unimplemented!() }

    fn xor<T>(&self)
    where T: Copy + BitXor { unimplemented!() }

    fn shl<T>(&self)
    where T: Copy + Shl { unimplemented!() }

    fn shr<T>(&self)
    where T: Copy + Shr { unimplemented!() }

    fn ushr<T>(&self)
    where T: Copy + Shr<jint> {
        let slots = cal_slots::<T>();

        let value2 = self._stack.pop::<jint>(cal_slots::<jint>());
        let value1 = self._stack.pop::<T>(slots);

        let result = value1 >> value2;

        self._stack.push(slots, result);
    }

    fn cmp_floating<T>(&self, res_for_nan: i32)
    where T: Copy + Float {
        let slots = cal_slots::<T>();

        let value2 = self._stack.pop::<T>(slots);
        let value1 = self._stack.pop::<T>(slots);

        let mut res: jint = 0;

        if value1.is_nan() || value2.is_nan() {
            res = res_for_nan;
        } else if value1 > value2 {
            res = 1;
        } else if value2 == value2 {
            res = 0;
        } else {
            res = -1;
        }

        self._stack.push(slots, res);
    }

    fn cmpg<T>(&self)
    where T: Copy + Float {
        self.cmp_floating::<T>(1);
    }

    fn cmpl<T>(&self)
    where T: Copy + Float {
        self.cmp_floating::<T>(-1);
    }
}

impl<'a> Executor<'a> {
    pub fn interpret(&self) -> Result<(), String> {
        let mut code = self.code();
        let rpc = &mut self.regs().pc;

        loop {
            let opc = &code.opcodes[*rpc as usize];
            match &opc.1 {
                Opcode::Aaload => {
                    self.array_load_ptr();
                }

                Opcode::Aastore => {
                    self.array_store_ptr();
                }

                Opcode::AconstNull => {
                    self._stack.push::<address>(SLOT_PER_PTR, 0);
                }

                Opcode::Aload(index) => {
                    self.local_load_ptr(*index);
                }

                Opcode::Anewarray(t) => {
                    // ...
                } 

                Opcode::Areturn => {
                    if self.return_with_value::<address>(SLOT_PER_PTR) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Arraylength => {
                    || -> Option<()> {
                        let arrayref = self.pop_valid_obj::<ArrayObjDesc>()?;
                        let value = arrayref.length();
                        self._stack.push(cal_slots::<jint>(), value);

                        None
                    } ();
                }

                Opcode::Astore(index) => {
                    self.local_store_ptr(*index);
                }

                Opcode::Athrow => {
                    // ...
                }

                Opcode::Baload => {
                    self.array_load::<jbyte>();
                }

                Opcode::Bastore => {
                    self.array_store::<jbyte>();
                }

                Opcode::Bipush(byte) => {
                    self._stack.push(cal_slots::<jint>(), *byte as jint);
                }

                Opcode::Caload => {
                    self.array_load::<jchar>();
                }

                Opcode::Castore => {
                    self.array_store::<jchar>();
                }

                Opcode::Checkcast(t) => {
                    // ...
                }

                Opcode::D2f => {
                    self.primitive_cast::<jdouble, jfloat>();
                }

                Opcode::D2i => {
                    self.primitive_cast::<jdouble, jint>();
                }

                Opcode::D2l => {
                    self.primitive_cast::<jdouble, jlong>();
                }

                Opcode::Dadd => {
                    self.add::<jdouble>();
                }

                Opcode::Daload => {
                    self.array_load::<jdouble>();
                }

                Opcode::Dastore => {
                    self.array_store::<jdouble>();
                }

                Opcode::Dcmpg => {
                    self.cmpg::<jdouble>();
                }

                Opcode::Dcmpl => {
                    self.cmpl::<jdouble>();
                }

                Opcode::Dconst0 => {
                    self._stack.push(cal_slots::<jdouble>(), 0.0f64);
                }

                Opcode::Dconst1 => {
                    self._stack.push(cal_slots::<jdouble>(), 1.0f64);
                }

                Opcode::Ddiv => {
                    self.div::<jdouble>();
                }

                Opcode::Dload(index) => {
                    self.local_load::<jdouble>(*index);
                }

                Opcode::Dmul => {
                    self.mul::<jdouble>();
                }

                Opcode::Dneg => {
                    self.neg::<jdouble>();
                }

                Opcode::Drem => {
                    self.rem::<jdouble>();
                }

                Opcode::Dreturn => {
                    if self.return_with_value::<jdouble>(cal_slots::<jdouble>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Dstore(index) => {
                    self.local_store::<jdouble>(*index);
                }

                Opcode::Dsub => {
                    self.sub::<jdouble>();
                }

                Opcode::Dup => {
                    self.dup();
                }

                Opcode::DupX1 => {
                    self.dup_x1();
                }

                Opcode::DupX2 => {
                    self.dup_x2();
                }

                Opcode::Dup2 => {
                    self.dup2();
                }

                Opcode::Dup2X1 => {
                    self.dup2_x1();
                }

                Opcode::Dup2X2 => {
                    self.dup2_x2();
                }

                Opcode::F2d => {
                    self.primitive_cast::<jfloat, jdouble>();
                }

                Opcode::F2i => {
                    self.primitive_cast::<jfloat, jint>();
                }

                Opcode::F2l => {
                    self.primitive_cast::<jfloat, jlong>();
                }

                Opcode::Fadd => {
                    self.add::<jfloat>();
                }

                Opcode::Faload => {
                    self.array_load::<jfloat>();
                }

                Opcode::Fastore => {
                    self.array_store::<jfloat>();
                }

                Opcode::Fcmpg => {
                    self.cmpg::<jfloat>();
                }

                Opcode::Fcmpl => {
                    self.cmpl::<jfloat>();
                }

                Opcode::Fconst0 => {
                    self._stack.push(cal_slots::<jfloat>(), 0.0f32);
                }

                Opcode::Fconst1 => {
                    self._stack.push(cal_slots::<jfloat>(), 1.0f32);
                }

                Opcode::Fconst2 => {
                    self._stack.push(cal_slots::<jfloat>(), 2.0f32);
                }

                Opcode::Fdiv => {
                    self.div::<jfloat>();
                }

                Opcode::Fload(index) => {
                    self.local_load::<jfloat>(*index);
                }

                Opcode::Fmul => {
                    self.mul::<jfloat>();
                }

                Opcode::Fneg => {
                    self.neg::<jfloat>();
                }

                Opcode::Frem => {
                    self.rem::<jfloat>();
                }

                Opcode::Freturn => {
                    if self.return_with_value::<jfloat>(cal_slots::<jfloat>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Fstore(index) => {
                    self.local_store::<jfloat>(*index);
                }

                Opcode::Fsub => {
                    self.sub::<jfloat>();
                }

                Opcode::Getfield(field_ref) => {
                    // ...
                }

                Opcode::Getstatic(field_ref) => {
                    // ...
                }

                Opcode::Goto(offset) => {
                    // ...
                }

                Opcode::I2b => {
                    self.primitive_cast::<jint, jbyte>();
                }

                Opcode::I2c => {
                    self.primitive_cast::<jint, jchar>();
                }

                Opcode::I2d => {
                    self.primitive_cast::<jint, jdouble>();
                }

                Opcode::I2f => {
                    self.primitive_cast::<jint, jfloat>();
                }

                Opcode::I2l => {
                    self.primitive_cast::<jint, jlong>();
                }

                Opcode::I2s => {
                    self.primitive_cast::<jint, i16>();
                }

                Opcode::Iadd => {
                    self.add::<jint>();
                }

                Opcode::Iaload => {
                    self.array_load::<jint>();
                }

                Opcode::Iand => {
                    self.and::<jint>();
                }

                Opcode::Iastore => {
                    self.array_store::<jint>();
                }

                Opcode::Iconst0 => {
                    self._stack.push(cal_slots::<jint>(), 0 as jint);
                }

                Opcode::Iconst1 => {
                    self._stack.push(cal_slots::<jint>(), 1 as jint);
                }

                Opcode::Iconst2 => {
                    self._stack.push(cal_slots::<jint>(), 2 as jint);
                }

                Opcode::Iconst3 => {
                    self._stack.push(cal_slots::<jint>(), 3 as jint);
                }

                Opcode::Iconst4 => {
                    self._stack.push(cal_slots::<jint>(), 4 as jint);
                }

                Opcode::Iconst5 => {
                    self._stack.push(cal_slots::<jint>(), 5 as jint);
                }

                Opcode::IconstM1 => {
                    self._stack.push(cal_slots::<jint>(), -1 as jint);
                }

                Opcode::Idiv => {
                    self.div::<jint>();
                }

                Opcode::IfAcmpeq(offs) => {
                    // ...
                }

                Opcode::IfAcmpne(offs) => {
                    // ...
                }

                Opcode::IfIcmpeq(offs) => {
                    // ...
                }

                Opcode::IfIcmpne(offs) => {
                    // ...
                }

                Opcode::IfIcmplt(offs) => {
                    // ...
                }

                Opcode::IfIcmpge(offs) => {
                    // ...
                }
                
                Opcode::IfIcmpgt(offs) => {
                    // ...
                }

                Opcode::IfIcmple(offs) => {
                    // ...
                }

                Opcode::Ifeq(offs) => {
                    // ...
                }

                Opcode::Ifne(offs) => {
                    // ...
                }

                Opcode::Iflt(offs) => {
                    // ...
                }

                Opcode::Ifge(offs) => {
                    // ...
                }

                Opcode::Ifgt(offs) => {
                    // ...
                }

                Opcode::Ifle(offs) => {
                    // ...
                }

                Opcode::Ifnonnull(offs) => {
                    // ...
                }

                Opcode::Ifnull(offs) => {
                    // ...
                }

                Opcode::Iinc(index, con) => {
                    // ...
                }

                Opcode::Iload(index) => {
                    self.local_load::<jint>(*index);
                }

                Opcode::Imul => {
                    self.mul::<jint>();
                }

                Opcode::Ineg => {
                    self.neg::<jint>();
                }

                Opcode::Instanceof(t) => {
                    // ...
                }

                Opcode::Invokedynamic(t) => {
                    // ...
                }

                Opcode::Invokeinterface(r, count) => {
                    // ...
                }

                Opcode::Invokespecial(r) => {
                    // ...
                }

                Opcode::Invokestatic(r) => {
                    // ...
                }

                Opcode::Invokevirtual(r) => {
                    // ...
                }

                Opcode::Ior => {
                    self.or::<jint>();
                }

                Opcode::Irem => {
                    self.rem::<jint>();
                }

                Opcode::Ireturn => {
                    if self.return_with_value::<jint>(cal_slots::<jint>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Ishl => {
                    self.shl::<jint>();
                }

                Opcode::Ishr => {
                    self.shr::<jint>();
                }

                Opcode::Istore(index) => {
                    self.local_store::<jint>(*index);
                }

                Opcode::Isub => {
                    self.sub::<jint>();
                }

                Opcode::Iushr => {
                    self.ushr::<u32>();
                }

                Opcode::Ixor => {
                    self.xor::<jint>();
                }

                Opcode::Jsr(offs) => {
                    // ...
                }

                Opcode::L2d => {
                    self.primitive_cast::<jlong, jdouble>();
                }

                Opcode::L2f => {
                    self.primitive_cast::<jlong, jfloat>();
                }

                Opcode::L2i => {
                    self.primitive_cast::<jlong, jint>();
                }

                Opcode::Ladd => {
                    self.add::<jlong>();
                }

                Opcode::Laload => {
                    self.array_load::<jlong>();
                }

                Opcode::Land => {
                    self.and::<jlong>();
                }

                Opcode::Lastore => {
                    self.array_store::<jlong>();
                }

                Opcode::Lcmp => {
                    
                }

                Opcode::Lconst0 => {
                    self._stack.push(cal_slots::<jlong>(), 0 as jlong);
                }

                Opcode::Lconst1 => {
                    self._stack.push(cal_slots::<jlong>(), 1 as jlong);
                }

                Opcode::Ldc(i) => {
                    // ...
                }

                Opcode::LdcW(i) => {
                    // ...
                }

                Opcode::Ldc2W(i) => {
                    // ...
                }

                Opcode::Ldiv => {
                    self.div::<jlong>();
                }

                Opcode::Lload(index) => {
                    self.local_load::<jlong>(*index);
                }

                Opcode::Lmul => {
                    self.mul::<jlong>();
                }

                Opcode::Lneg => {
                    self.neg::<jlong>();
                }

                Opcode::Lookupswitch(t) => {
                    // ...
                }

                Opcode::Lor => {
                    self.or::<jlong>();
                }

                Opcode::Lrem => {
                    self.rem::<jlong>();
                }

                Opcode::Lreturn => {
                    if self.return_with_value::<jlong>(cal_slots::<jlong>()) {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Lshl => {
                    self.shl::<jlong>();
                }

                Opcode::Lshr => {
                    self.shr::<jlong>();
                }

                Opcode::Lstore(index) => {
                    self.local_store::<jlong>(*index);
                }

                Opcode::Lsub => {
                    self.sub::<jlong>();
                }

                Opcode::Lushr => {
                    self.ushr::<u64>();
                }

                Opcode::Lxor => {
                    self.xor::<jlong>();
                }

                Opcode::Monitorenter => {
                    // ...
                }

                Opcode::Monitorexit => {
                    // ...
                }

                Opcode::Multianewarray(t, dims) => {
                    // ...
                }

                Opcode::New(t) => {
                    // ...
                }

                Opcode::Newarray(t) => {
                    // ...
                }

                Opcode::Nop => {
                    // Do nothing
                }

                Opcode::Pop => {
                    self._stack.pop::<slot_t>(1);
                }

                Opcode::Pop2 => {
                    self._stack.pop::<slot_t>(2);
                }

                Opcode::Putfield(field_ref) => {
                    // ...
                }

                Opcode::Putstatic(field_ref) => {
                    // ...
                }

                Opcode::Ret(index) => {
                    // ...
                }

                Opcode::Return => {
                    if self.return_void() {
                        code = self.code();
                        continue;
                    }
                }

                Opcode::Saload => {
                    self.array_load::<i16>();
                }

                Opcode::Sastore => {
                    self.array_store::<i16>();
                }

                Opcode::Sipush(short) => {
                    self._stack.push(cal_slots::<jint>(), *short as jint);
                }

                Opcode::Swap => {
                    let value2 = self._stack.pop::<slot_t>(1);
                    let value1 = self._stack.pop::<slot_t>(1);
                    self._stack.push(1, value2);
                    self._stack.push(1, value1);
                }

                Opcode::Tableswitch(rt) => {
                    // ...
                }

                // reserved codes

                Opcode::Breakpoint => {
                    // ...
                }

                Opcode::Impdep1 => {
                    // ...
                }

                Opcode::Impdep2 => {
                    // ...
                }
            }

            *rpc += 1;
        }

        Ok(())
    }
}
