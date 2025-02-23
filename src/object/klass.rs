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

use cafebabe::ClassFile;

use crate::{class_loader::rtcp::RuntimeConstantPool, util::lock_free_stack::NextPtr};

pub struct Klass<'a> {
    // used for lock free stack
    _next_ptr: *const Klass<'a>,

    _class_file: ClassFile<'a>,
    _rtcp: RuntimeConstantPool,
}

impl<'a> NextPtr<Klass<'a>> for Klass<'a> {
    fn next_ptr(&mut self) -> *mut *const Klass<'a> {
        &mut self._next_ptr
    }
}
