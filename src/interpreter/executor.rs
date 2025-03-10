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

use std::u8;

use super::bytecodes::opcode;

pub struct Executor {
    _pc: u16,
    _sp: u16,
    _bp: u16
}

impl Executor {
    pub fn execute(&mut self, code: &[u8]) -> Option<String> {
        let rpc = &mut self._pc;
        let rsp = &mut self._sp;
        let rbp = &mut self._bp;

        loop {
            let opc = code[*rpc as usize];
            match opc {
                opcode::_NOP => {}
                opcode::_AALOAD => {}

                _ => { return Some(format!("illegal code: {:#X}", opc)); }
            }
        }
    }
}

// helpers
impl Executor {

}
