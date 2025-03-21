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

use std::cell::{RefCell, UnsafeCell};

use cafebabe::attributes::CodeData;

use crate::{memory::mem_region::MemRegion, runtime::{frame::Frame, vmflags::CompressedPtr}, util::global_defs::{addr_cast, address}};

pub struct InterpreterRegisters<'a> {
    pub(super) pc: u16,

    sp: address,
    bp: Option<&'a Frame<'a>>
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

impl<'a> InterpreterRegisters<'a> {
    // no range check
    fn alloca_sized<T: Sized>(&mut self) -> &'a mut T {
        self.sp -= size_of::<T>();

        unsafe { &mut *(self.sp as *mut _) }
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
    _regs: UnsafeCell<Option<&'a mut InterpreterRegisters<'a>>>,
    _mr: MemRegion
}

impl<'a> InterpreterStack<'a> {
    pub fn new() -> Self {
        Self {
            _regs: UnsafeCell::new(None),
            _mr: MemRegion::new()
        }
    }

    pub fn init(&mut self, size: usize, regs: &'a mut InterpreterRegisters<'a>) {
        self._regs = UnsafeCell::new(Some(regs));
        self._mr = MemRegion::with_size(Vec::<u8>::with_capacity(size).as_ptr() as _, size);

        self.regs().sp = self._mr.last_word();
    }
}

impl<'a> InterpreterStack<'a> {
    fn regs(&self) -> &mut InterpreterRegisters<'a> {
        unsafe {
            (*self._regs.get()).as_mut().unwrap()
        }
    }
}

impl<'a> InterpreterStack<'a> {
    fn assert_new_sp_available(&self, n: address) -> Result<(), String> {
        if !self._mr.contains(n) {
            return Err(format!("Segmentation fault."));
        }

        Ok(())
    }

    pub fn push<T: Sized>(&self, n: T) -> Result<(), String> {
        let new_sp = self.regs().sp - size_of::<T>();
        self.assert_new_sp_available(new_sp)?;
        self.regs().sp = new_sp;

        unsafe { *(new_sp as *mut _) = n };

        Ok(())
    }

    pub fn alloca(&self, s: usize) -> Result<address, String> {
        let new_sp = self.regs().sp - s;
        self.assert_new_sp_available(new_sp)?;
        self.regs().sp = new_sp;

        Ok(new_sp)
    }

    pub fn pop<T>(&self) -> Result<T, String>
    where T: Sized + Copy {
        let sp = self.regs().sp;
        let new_sp = sp + size_of::<T>();
        self.assert_new_sp_available(new_sp)?;
        self.regs().sp = new_sp;

        unsafe { Ok(*(sp as *const _)) }
    }

    pub fn push_ptr(&self, n: address) -> Result<(), String> {
        if CompressedPtr {
            self.push(n as u32)?
        } else {
            self.push(n)?
        }

        Ok(())
    }

    pub fn pop_ptr(&self) -> Result<address, String> {
        if CompressedPtr {
            Ok(self.pop::<u32>()? as _)
        } else {
            Ok(self.pop()?)
        }
    }

    pub fn create_frame(&self, cd: &'a CodeData) -> Result<(), String> {
        let f = addr_cast::<Frame>(self.alloca(size_of::<Frame>())?);
        f.init(self.regs().clone(), cd);

        self.regs().bp = Some(f);

        Ok(())
    }

    pub fn unwind(&self) -> Option<&CodeData> {
        match self.regs().bp {
            Some(x) => {
                *self.regs() = x.last_regs();
                Some(x.last_code_data())
            },

            None => None
        }
    }
}
