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

#[macro_export]
macro_rules! OneBit {
    () => { 0x1 }
}

pub type intx = isize;
pub type uintx = usize;

pub type word_t = uintx;
pub type address = uintx;

pub const LOG_BITS_PER_BYTE: i32 = 3;
pub const BITS_PER_BYTE: i32 = OneBit!() << LOG_BITS_PER_BYTE;

pub const LOG_BYTES_PER_SHORT: i32 = 1;
pub const LOG_BYTES_PER_INT: i32 = 2;
pub const LOG_BYTES_PER_ARCH: i32 = 3;

pub const BYTES_PER_SHORT: i32 = OneBit!() << LOG_BYTES_PER_SHORT;
pub const BYTES_PER_ARCH: i32 = OneBit!() << LOG_BYTES_PER_ARCH;

pub const LOG_BITS_PER_ARCH: i32 = LOG_BITS_PER_BYTE + LOG_BYTES_PER_ARCH;

pub const BITS_PER_ARCH: i32 = OneBit!() << LOG_BITS_PER_ARCH;

