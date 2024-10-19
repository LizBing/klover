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

use std::io;

pub struct ClassFileStream {
    _buffer: Vec<u8>,
    _cursor: usize,
}

impl ClassFileStream {
    pub fn new(buffer: Vec<u8>) -> ClassFileStream {
        ClassFileStream {
            _buffer: buffer,
            _cursor: 0
        }
    }

    fn get<T: Copy>(&mut self) -> io::Result<T> {
        let off = self._cursor;
        self._cursor += size_of::<T>();

        if off >= self._buffer.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data"))
        }

        let ptr = &self._buffer[off] as *const u8 as *const T;
        unsafe { Ok(*ptr) }
    }

    pub fn get_u1(&mut self) -> io::Result<u8> {
        self.get::<u8>()
    }

    pub fn get_u2(&mut self) -> io::Result<u16> {
        self.get::<u16>()
    }

    pub fn get_u4(&mut self) -> io::Result<u32> {
        self.get::<u32>()
    }

    pub fn get_byte_array(&mut self, size: usize) -> io::Result<Vec<u8>> {
        let off = self._cursor;
        self._cursor += size;

        if off >= self._buffer.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data"))
        }

        let mut res = Vec::new();
        res.copy_from_slice(&self._buffer[off..size]);

        Ok(res)
    }
}
