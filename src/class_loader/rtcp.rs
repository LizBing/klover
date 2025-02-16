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

use bit_set::BitSet;

pub enum RtConstantPoolEntry {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    ClassRef(String),
    // Additional constant types as needed
}

pub struct RuntimeConstantPool {
    constants: Vec<RtConstantPoolEntry>,
    resolved: BitSet,
}

impl RuntimeConstantPool {
    pub fn new(capacity: usize) -> Self {
        RuntimeConstantPool {
            constants: Vec::with_capacity(capacity),
            resolved: BitSet::with_capacity(capacity),
        }
    }

    pub fn add_entry(&mut self, entry: RtConstantPoolEntry) -> usize {
        let index = self.constants.len();
        self.constants.push(entry);
        index
    }

    pub fn get_entry(&self, index: usize) -> Option<&RtConstantPoolEntry> {
        self.constants.get(index)
    }

    pub fn mark_resolved(&mut self, index: usize) {
        self.resolved.insert(index);
    }

    pub fn is_resolved(&self, index: usize) -> bool {
        self.resolved.contains(index)
    }
}
