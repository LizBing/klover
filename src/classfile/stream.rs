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

use std::io::{self, Cursor, Read};
use byteorder::{NetworkEndian, ReadBytesExt};

pub struct ClassFileStream {
    _cursor: Cursor<Vec<u8>>,
}

impl ClassFileStream {
    pub fn new(buffer: Vec<u8>) -> ClassFileStream {
        ClassFileStream {
            _cursor: Cursor::new(buffer)
        }
    }

    pub fn get_u1(&mut self) -> io::Result<u8> {
        self._cursor.read_u8()
    }

    pub fn get_u2(&mut self) -> io::Result<u16> {
        self._cursor.read_u16::<NetworkEndian>()
    }

    pub fn get_u4(&mut self) -> io::Result<u32> {
        self._cursor.read_u32::<NetworkEndian>()
    }

    pub fn get_byte_array(&mut self, size: usize) -> io::Result<Vec<u8>> {
        let mut res = Vec::new();
        res.reserve_exact(size);
        unsafe { res.set_len(size) };

        self._cursor.read_exact(&mut res[..size])?;
        Ok(res)
    }
}
