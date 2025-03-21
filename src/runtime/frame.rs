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

use cafebabe::attributes::CodeData;

use crate::interpreter::interpreter_runtime::InterpreterRegisters;

pub struct Frame<'a> {
    _last_regs: InterpreterRegisters<'a>,
    _last_code_data: &'a CodeData<'a>
}

impl<'a> Frame<'a> {
    pub fn init(&mut self, regs: InterpreterRegisters<'a>, cd: &'a CodeData) {
        *self = Self {
            _last_regs: regs,
            _last_code_data: cd
        }
    }

    pub fn last_regs(&self) -> InterpreterRegisters {
        self._last_regs.clone()
    }

    pub fn last_code_data(&self) -> &CodeData {
        self._last_code_data
    }
}

impl<'a> Drop for Frame<'a> {
    fn drop(&mut self) {
        panic!("Should not reach here.")
    }
}
