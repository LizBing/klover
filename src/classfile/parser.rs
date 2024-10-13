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

use super::{constant_pool::ConstantPool, info_types::{AttributeInfo, FieldInfo, MethodInfo}, stream::ClassFileStream};
use super::{MAGIC, SMALLEST_MAJOR, LARGEST_MAJOR, SPECIFIC_MINOR_0, SPECIFIC_MINOR_65535, GENERAL_MAJOR};

pub struct ClassFileParser {
    _access_flag:   u16,
    _this_class:    u16,
    _super_class:   u16,

    _cp:            ConstantPool,

    _field_infos:   Vec<FieldInfo>,
    _mthd_infos:    Vec<MethodInfo>,
    _interfaces:    Vec<u16>,
    _attr_infos:    Vec<AttributeInfo>,
}

impl ClassFileParser {
    pub fn new() -> ClassFileParser {
        ClassFileParser {
            _access_flag: 0,
            _this_class: 0,
            _super_class: 0,

            _cp: ConstantPool {},

            _field_infos: Vec::new(),
            _mthd_infos: Vec::new(),
            _interfaces: Vec::new(),
            _attr_infos: Vec::new()
        }
    }

    pub fn parse(&mut self, stream: &mut ClassFileStream) -> io::Result<Option<String>> {
        let load_magic = stream.get_u4()?;
        if let Some(x) = ClassFileParser::verify_magic(load_magic) {
            return Ok(Some(x));
        }

        let load_minor = stream.get_u2()?;
        let load_major = stream.get_u2()?;
        if let Some(x) = ClassFileParser::verify_version(load_minor, load_major) {
            return Ok(Some(x));
        }

        Ok(None)
    }
    
    fn verify_magic(n: u32) -> Option<String> {
        if n != MAGIC {
            return Some(format!("Invaild magic: {}. Should be 0x{:X}", n, MAGIC));
        }

        None
    }

    fn verify_version(minor: u16, major: u16) -> Option<String> {
        if major < SMALLEST_MAJOR || major > LARGEST_MAJOR {
            return Some(format!("Invaild major version: {}. Should be [{}, {}].",
                                major, SMALLEST_MAJOR, LARGEST_MAJOR));
        } else if major >= GENERAL_MAJOR && 
                  (minor != SPECIFIC_MINOR_0 || minor != SPECIFIC_MINOR_65535) {
            return Some(format!("Invaild minor version: {}. Should be {} or {}.",
                                minor, SPECIFIC_MINOR_0, SPECIFIC_MINOR_65535));
        }

        None
    }
}
