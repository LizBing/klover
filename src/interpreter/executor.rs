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

use std::cell::UnsafeCell;

use cafebabe::{attributes::CodeData, bytecode::Opcode};

use crate::{ jni::{jbyte, jchar, jdouble, jfloat, jint, jlong}, object::{klass::Klass, obj_desc::ArrayObjDesc}, util::global_defs::{addr_cast, address}};

use super::interpreter_runtime::{InterpreterRegisters, InterpreterStack};

pub struct Executor<'a> {
    _regs: UnsafeCell<InterpreterRegisters<'a>>,
    _code_data: &'a CodeData<'a>,
    _stack: InterpreterStack<'a>,
}

impl<'a> Executor<'a> {
    pub fn new(cd: &'a CodeData) -> Self {
        Self {
            _regs: UnsafeCell::new(InterpreterRegisters::new()),
            _code_data: cd,
            _stack: InterpreterStack::new()
        }
    }

    pub fn init(&'a mut self, stack_size: usize) {
        self._stack.init(stack_size, self._regs.get_mut());
    }
}

impl<'a> Executor<'a> {
    fn regs(&self) -> &mut InterpreterRegisters<'a> {
        unsafe { &mut *(self._regs.get()) }
    }
}

impl<'a> Executor<'a> {
    fn throw_exception(&mut self, e: &Klass) {
        unimplemented!()
    }

    fn pop_valid_ptr(&self) -> address {
        unimplemented!()
    }

    fn array_load<T>(&self) { unimplemented!() }

    fn array_load_ptr(&self) { unimplemented!() }

    fn array_store<T>(&self) { unimplemented!() }

    fn array_store_ptr(&self) { unimplemented!() }

    fn local_load<T>(&self, index: u16) { unimplemented!() }

    fn local_load_ptr(&self, index: u16) { unimplemented!() }

    fn local_store<T>(&self, index: u16) { unimplemented!() }

    fn local_store_ptr(&self, index: u16) { unimplemented!() }

    fn return_v<T>(&self) { unimplemented!() }

    fn return_ptr(&self) { unimplemented!() }

    fn dup(&self) { unimplemented!() }

    fn dup_x1(&self) { unimplemented!() }

    fn dup_x2(&self) { unimplemented!() }

    fn dup2(&self) { unimplemented!() }

    fn dup2_x1(&self) { unimplemented!() }

    fn dup2_x2(&self) { unimplemented!() }

    fn primitive_cast<FromType, ToType>(&self) { unimplemented!() }

    fn add<T>(&self) { unimplemented!() }

    fn sub<T>(&self) { unimplemented!() }

    fn mul<T>(&self) { unimplemented!() }

    fn div<T>(&self) { unimplemented!() }

    fn neg<T>(&self) { unimplemented!() }

    fn rem<T>(&self) { unimplemented!() }

    fn cmpg<T>(&self) { unimplemented!() }

    fn cmpl<T>(&self) { unimplemented!() }
}

impl<'a> Executor<'a> {
    pub fn execute(&self) -> Result<(), String> {
        let code = self._code_data.bytecode.as_ref().unwrap();
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
                    self._stack.push_ptr(0);
                }

                Opcode::Aload(index) => {
                    self.local_load_ptr(*index);
                }

                Opcode::Anewarray(t) => {
                    // ...
                } 

                Opcode::Areturn => {
                    self.return_ptr();
                }

                Opcode::Arraylength => {
                    let arrayref = self.pop_valid_ptr();

                    // barrier

                    let arr = addr_cast::<ArrayObjDesc>(arrayref);
                    self._stack.push(arr.length());
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
                    self._stack.push(*byte as jint);
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
                    self._stack.push(0.0f64);
                }

                Opcode::Dconst1 => {
                    self._stack.push(1.0f64);
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
                    self.return_v::<jdouble>();
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
                    self._stack.push(0.0f32);
                }

                Opcode::Fconst1 => {
                    self._stack.push(1.0f32);
                }

                Opcode::Fconst2 => {
                    self._stack.push(2.0f32);
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
                    self.return_v::<jfloat>();
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
                    let value2 = self._stack.pop::<jint>();
                    let value1 = self._stack.pop::<jint>();
                    self._stack.push(value1 & value2);
                }

                Opcode::Iastore => {
                    self.array_store::<jint>();
                }

                Opcode::Iconst0 => {
                    self._stack.push(0 as jint);
                }

                Opcode::Iconst1 => {
                    self._stack.push(1 as jint);
                }

                Opcode::Iconst2 => {
                    self._stack.push(2 as jint);
                }

                Opcode::Iconst3 => {
                    self._stack.push(3 as jint);
                }

                Opcode::Iconst4 => {
                    self._stack.push(4 as jint);
                }

                Opcode::Iconst5 => {
                    self._stack.push(5 as jint);
                }

                Opcode::IconstM1 => {
                    self._stack.push(-1 as jint);
                }

                Opcode::Idiv => {
                    self.div::<jint>();
                }

                Opcode::Iinc(index, con) => {
                    // ...
                }

                // ... more opcodes would continue here
                
                _ => break,
            }

            *rpc += 1;
        }

        Ok(())
    }
}
