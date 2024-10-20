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

use std::{fs::File, io::{self, Read}};

use super::{parser::ClassFileParser, stream::ClassFileStream};

pub struct ClassLoader {
    _file_path: String,
    _stream: ClassFileStream,
}

impl ClassLoader {
    pub fn new(path: String) -> io::Result<ClassLoader> {
        let mut file = File::open(&path)?;

        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer)?;

        Ok(ClassLoader {
            _file_path: path,
            _stream: ClassFileStream::new(buffer)
        })
    }

    pub fn parse(&mut self) -> io::Result<Result<ClassFileParser, String>> {
        let mut parser = ClassFileParser::new();
        if let Some(x) = parser.parse(&mut self._stream)? {
            return Ok(Err(x))
        }

        Ok(Ok(parser))
    }
}
