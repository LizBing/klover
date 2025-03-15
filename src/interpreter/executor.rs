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

use std::{ptr::null, u8};

use cafebabe::{attributes::CodeData, bytecode::Opcode};

use crate::{runtime::frame::Frame, util::global_defs::address};

pub struct InterpreterRegisters<'a> {
    _pc: u16,
    _sp: address,
    _bp: Option<&'a Frame<'a>>
}

impl<'a> InterpreterRegisters<'a> {
    fn new() -> Self {
        Self {
            _pc: 0,
            _sp: 0,
            _bp: None
        }
    }
}

impl<'a> InterpreterRegisters<'a> {
    fn alloca_sized<T: Sized>(&mut self) -> &'a mut T {
        unimplemented!()
    }
}

impl<'a> Clone for InterpreterRegisters<'a> {
    fn clone(&self) -> Self {
        Self {
            _pc: self._pc,
            _sp: self._sp,
            _bp: self._bp
        }
    }
}

pub struct Executor<'a> {
    _registers: InterpreterRegisters<'a>,
    _code_data: &'a CodeData<'a>,
    _stack: Box<[u8]>
}

impl<'a> Executor<'a> {
    pub fn new(cd: &CodeData) -> Result<Self, String> {
        unimplemented!()
    }
}

// forwarding
impl<'a> Executor<'a> {
    fn rpc(&mut self) -> &mut u16 {
        &mut self._registers._pc
    }

    fn rsp(&mut self) -> &mut address {
        &mut self._registers._sp
    }

    fn rbp(&mut self) -> &mut Option<&'a Frame<'a>> {
        &mut self._registers._bp
    }
}

impl<'a> Executor<'a> {
    pub fn execute(&mut self) -> Option<String> {
        let code = self._code_data.bytecode.as_ref().unwrap();
        loop {
            let opc = &code.opcodes[*self.rpc() as usize];
            match opc.1 {
                Opcode::Aaload => { () }

                _ => break,
            }

            *self.rpc() += 1;
        }

        None
    }
}

impl<'a> Executor<'a> {
    fn create_frame(&mut self) {
        let tmp_regs = self._registers.clone();
        let frame = self._registers.alloca_sized::<Frame>();
        frame.init(tmp_regs, self._code_data);

        *self.rbp() = Some(frame);
    }

    fn unwind(&mut self) -> bool {
        match *self.rbp() {
            Some(x) => {
                self._registers = x.last_regs();
                self._code_data = x.last_code_data();

                true
            }

            None => false
        }
    }
}
