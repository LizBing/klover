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

use crate::{ object::{klass::Klass, obj_desc::ArrayObjDesc}, util::global_defs::{addr_cast, address}};

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
}

impl<'a> Executor<'a> {
    pub fn execute(&self) -> Result<(), String> {
        let code = self._code_data.bytecode.as_ref().unwrap();
        let rpc = &mut self.regs().pc;

        loop {
            let opc = &code.opcodes[*rpc as usize];
            match &opc.1 {
                Opcode::Aaload => {
                    let index = self._stack.pop();
                    let arrayref = self.pop_valid_ptr();

                    // barrier

                    let arr = addr_cast::<ArrayObjDesc>(arrayref);
                    let value = arr.get(index);
                    self._stack.push_ptr(value);
                }

                Opcode::Aastore => {
                    let value = self._stack.pop();
                    let index = self._stack.pop();
                    let arrayref = self.pop_valid_ptr();

                    // barrier

                    let arr = addr_cast::<ArrayObjDesc>(arrayref);
                    arr.put::<address>(index, value);
                }

                Opcode::AconstNull => {
                    self._stack.push(0 as address);
                }

                Opcode::Aload(index) => {
                    // resolve oop map
                    // ...
                }

                Opcode::Anewarray(t) => {
                    // ...
                } 

                Opcode::Areturn => {
                    // ...
                }

                Opcode::Arraylength => {
                    let arrayref = self.pop_valid_ptr();

                    // null check
                    // barrier

                    let arr = addr_cast::<ArrayObjDesc>(arrayref);
                    self._stack.push(arr.length());
                }

                Opcode::Astore(index) => {
                    // resolve oop map
                    // ...
                }

                Opcode::Athrow => {
                    // ...
                }

                _ => break,
            }

            *rpc += 1;
        }

        Ok(())
    }
}
