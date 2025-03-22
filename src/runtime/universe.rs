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

use std::borrow::Cow;

use dashmap::{DashMap, DashSet};

use crate::{metaspace::klass_mem_pool::KlassMemPool, object::klass::Klass, util::global_defs::{address, naddr}};

pub struct Universe<'a> {
    _klass_mem_pool: KlassMemPool<'a>,
    _str_pool: DashSet<Cow<'a, str>>,
    _klasses: DashMap<Cow<'a, str>, Box<Klass<'a>>>
}

impl<'a> Universe<'a> {
    pub fn new() -> Result<Self, String> {
        Err(String::new())
    }
}

impl<'a> Universe<'a> {
    pub fn reslove_compressed_ptr(n: naddr) -> address {
        unimplemented!()
    }

    pub fn compress_ptr(n: address) -> naddr {
        unimplemented!()
    }
}
