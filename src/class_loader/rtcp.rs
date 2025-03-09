/*
 * Copyright (c) 2024, Lei Zaakjyu. All rights reserved.
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

// Runtime Constant Pool

use std::borrow::Cow;

use bit_set::BitSet;

use crate::object::klass::Klass;

pub enum RtConstantPoolEntry<'a> {
    Utf8(Cow<'a, str>),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ClassRef(Box<Klass<'a>>),
    // Additional constant types as needed
}

pub struct RuntimeConstantPool<'a> {
    _constants: Vec<RtConstantPoolEntry<'a>>,
    _resolved: BitSet,
}

impl<'a> RuntimeConstantPool<'a> {
    pub fn new(capacity: usize) -> Self {
        RuntimeConstantPool {
            _constants: Vec::with_capacity(capacity),
            _resolved: BitSet::with_capacity(capacity),
        }
    }

    pub fn add_entry(&mut self, index: usize, entry: RtConstantPoolEntry<'a>) {
        self._constants[index] = entry;
        self.mark_resolved(index);
    }

    pub fn get_entry(&self, index: usize) -> &RtConstantPoolEntry {
        &self._constants[index]
    }

    pub fn mark_resolved(&mut self, index: usize) {
        self._resolved.insert(index);
    }

    pub fn is_resolved(&self, index: usize) -> bool {
        self._resolved.contains(index)
    }
}
