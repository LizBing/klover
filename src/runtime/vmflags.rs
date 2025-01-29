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
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
 * OF ANY KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::os::raw::c_void;
use phf::{phf_map, phf_set, Map, Set};


type int = i32;
type intx = isize;
type uint = u32;
type uintx = usize;
type uint64_t = u64;
type size_t = usize;
type double = f64;


pub struct VMFlag {
    _name: &'static str,
    _type: &'static str,
    _desc: &'static str,
    _addr: *mut c_void
}

unsafe impl Sync for VMFlag {}

include!("vmflag_map.rs");
