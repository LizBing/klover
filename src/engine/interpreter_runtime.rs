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

use std::cell::{Cell, UnsafeCell};
use cafebabe::attributes::CodeData;
use crate::{align_up, code::method::Method, memory::mem_region::MemRegion, runtime::frame::Frame, util::global_defs::{addr_cast, address}};

use super::STACK_SLOT_SIZE;

pub struct InterpreterRegisters<'a> {
    pub(super) pc: u16,

    sp: address,
    bp: Option<&'a Frame<'a>>,
}

impl<'a> InterpreterRegisters<'a> {
    pub(super) fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            bp: None
        }
    }
}

impl<'a> Clone for InterpreterRegisters<'a> {
    fn clone(&self) -> Self {
        Self {
            pc: self.pc,
            sp: self.sp,
            bp: self.bp
        }
    }
}

pub(super) struct InterpreterStack<'a> {
    _regs: Option<&'a UnsafeCell<InterpreterRegisters<'a>>>,
    _mr: MemRegion,

    _locals: Cell<address>,
}

impl<'a> InterpreterStack<'a> {
    pub fn new() -> Self {
        Self {
            _regs: None,
            _mr: MemRegion::new(),
            _locals: Cell::new(0),
        }
    }

    pub fn init(&mut self, size: usize, regs: &'a UnsafeCell<InterpreterRegisters<'a>>) {
        self._regs = Some(regs);
        self._mr = MemRegion::with_size(Vec::<u8>::with_capacity(size).as_ptr() as _, size);

        self.regs().sp = self._mr.last_word();
    }
}

impl<'a> InterpreterStack<'a> {
    fn regs(&self) -> &mut InterpreterRegisters<'a> {
        unsafe {
            &mut *(self._regs.unwrap().get())
        }
    }
}

impl<'a> InterpreterStack<'a> {
    fn assert_entry_available(&self, n: address) -> Result<(), String> {
        if !self._mr.contains(n) {
            return Err(format!("Segmentation fault."));
        }

        Ok(())
    }

    pub fn push<T>(&self, slots: usize, n: T) {
        let new_sp = self.regs().sp - STACK_SLOT_SIZE * slots;
        self.regs().sp = new_sp;

        unsafe { *(new_sp as *mut _) = n };
    }

    pub fn alloca(&self, s: usize) -> Result<address, String> {
        let size = align_up!(s, STACK_SLOT_SIZE);
        let new_sp = self.regs().sp - size;
        self.assert_entry_available(new_sp)?;
        self.regs().sp = new_sp;

        Ok(new_sp)
    }

    pub fn pop<T: Copy>(&self, slots: usize) -> T {
        let old_sp = self.regs().sp;
        self.regs().sp = old_sp + STACK_SLOT_SIZE * slots;

        unsafe { *(old_sp as *const _) }
    }

    fn cal_addr_of_local(&self, index: u16) -> address {
        self._locals.get() + STACK_SLOT_SIZE * index as usize
    }

    pub fn load_local<T: Copy>(&self, index: u16) -> T {
        *addr_cast(self.cal_addr_of_local(index))
    }

    pub fn store_local<T>(&self, index: u16, n: T) {
        *addr_cast(self.cal_addr_of_local(index)) = n
    }

    // helper functions

    // We may waste a little memory if the CompressedPtr flag has not been set.
    fn cal_mem_size_of_locals(cd: &'a CodeData) -> usize {
        STACK_SLOT_SIZE * cd.max_locals as usize
    }

    fn cal_frame_size(cd: &'a CodeData) -> usize {
        size_of::<Frame>() + Self::cal_mem_size_of_locals(cd)
    }

    fn cal_mem_size_of_opstack(cd: &'a CodeData) -> usize {
        STACK_SLOT_SIZE * cd.max_stack as usize
    }

    fn set_base_of_locals(&self, bp: &Frame, cd: &'a CodeData) {
        self._locals.set(bp as *const _ as address - Self::cal_mem_size_of_locals(cd));
    }

    // We merely assume that the opcode is safe to run, so we do the assertion
    // of never-overflowing here.
    pub fn create_frame(&self, mthd: &'a Method) -> Result<(), String> {
        let old_regs = self.regs().clone();
        let cd = mthd.code_data().unwrap();

        let f = addr_cast::<Frame>(self.alloca(Self::cal_frame_size(cd))?);
        f.init(old_regs, mthd);

        // assert frame available
        let frame_edge = self.regs().sp - Self::cal_mem_size_of_opstack(cd);
        self.assert_entry_available(frame_edge)?;

        self.regs().bp = Some(f);
        self.set_base_of_locals(f, cd);

        Ok(())
    }

    // including compiled code
    pub fn create_frame_for_native(&self, mthd: &'a Method) -> Result<(), String> {
        let old_regs = self.regs().clone();

        let f = addr_cast::<Frame>(self.alloca(size_of::<Frame>())?);
        f.init(old_regs, mthd);

        self.regs().bp = Some(f);
        
        Ok(())
    }

    pub fn unwind(&self) -> Option<&'a Method<'a>> {
        match self.regs().bp {
            Some(x) => {
                let mthd = x.last_method();
                *self.regs() = x.last_regs();
                self.set_base_of_locals(x, mthd.code_data().unwrap());

                Some(mthd)
            },

            None => None
        }
    }
}
