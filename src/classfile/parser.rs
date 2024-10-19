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

use super::{constant_pool::ConstantPool, info_types::{AttributeInfo, FieldInfo, MethodInfo}, stream::ClassFileStream, MAJOR_RANGE};
use super::{MAGIC, SMALLEST_MAJOR, LARGEST_MAJOR, SPECIFIC_MINOR_0, SPECIFIC_MINOR_65535, GENERAL_MAJOR};
use super::constant_pool::*;
use super::constant_pool::ConstantPoolEntry::*;

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

            _cp: ConstantPool::new(),

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

        self.load_constants(stream);

        Ok(None)
    }
    
    fn verify_magic(n: u32) -> Option<String> {
        if n != MAGIC {
            return Some(format!("Invaild magic: {}. Should be 0x{:X}", n, MAGIC));
        }

        None
    }

    fn verify_version(minor: u16, major: u16) -> Option<String> {
        if !MAJOR_RANGE.contains(&major) {
            return Some(format!("Invaild major version: {}. Should be [{}, {}].",
                                major, SMALLEST_MAJOR, LARGEST_MAJOR));
        } else if major >= GENERAL_MAJOR && 
                  (minor != SPECIFIC_MINOR_0 || minor != SPECIFIC_MINOR_65535) {
            return Some(format!("Invaild minor version: {}. Should be {} or {}.",
                                minor, SPECIFIC_MINOR_0, SPECIFIC_MINOR_65535));
        }

        None
    }

    // Loading only, no verification.
    fn load_constants(&mut self, stream: &mut ClassFileStream) 
        -> io::Result<Option<String>> 
    {
        let constants = stream.get_u2()?;
        self._cp.reserve(constants as usize);

        for _ in 0..constants {
            let tag = stream.get_u1()?;
            match tag {
                CONSTANT_CLASS_TAG => {
                    self._cp.push(ConstantClassInfo(stream.get_u2()?));
                }

                CONSTANT_FIELDREF_TAG => {
                    self._cp.push(ConstantFieldrefInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_METHODREF_TAG => {
                    self._cp.push(ConstantMethodrefInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_INTERFACE_METHODREF_TAG => {
                    self._cp.push(ConstantInterfaceMethodrefInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_STRING_TAG => {
                    self._cp.push(ConstantStringInfo(stream.get_u2()?));
                }

                CONSTANT_INTEGER_TAG => {
                    self._cp.push(ConstantIntegerInfo(stream.get_u4()?));
                }

                CONSTANT_FLOAT_TAG => {
                    let u4form = stream.get_u4()?;
                    let f4form = unsafe { *(&u4form as *const u32 as *const f32) };
                    self._cp.push(ConstantFloatInfo(f4form));
                }

                CONSTANT_LONG_TAG => {
                    let high_bytes = stream.get_u4()?;
                    let low_bytes = stream.get_u4()?;
                    
                    let res = ((high_bytes as u64) << 32) + low_bytes as u64;
                    self._cp.push(ConstantLongInfo(res));
                    self._cp.push(ConstantDummyInfo);
                }

                CONSTANT_DOUBLE_TAG => {
                    let high_bytes = stream.get_u4()?;
                    let low_bytes = stream.get_u4()?;

                    let u8form = ((high_bytes as u64) << 32) + low_bytes as u64;
                    let res = unsafe { *(&u8form as *const u64 as *const f64) };
                    self._cp.push(ConstantDoubleInfo(res));
                    self._cp.push(ConstantDummyInfo);
                }

                CONSTANT_NAME_AND_TYPE_TAG => {
                    self._cp.push(ConstantNameAndTypeInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_UTF8_TAG => {
                    let length = stream.get_u2()? as usize;
                    let bytes = stream.get_byte_array(length)?;
                    unsafe { self._cp.push(ConstantUTF8Info(String::from_utf8_unchecked(bytes))); }
                }

                CONSTANT_METHOD_HANDLE_TAG => {
                    self._cp.push(ConstantMethodHandleInfo(stream.get_u1()?, stream.get_u2()?));
                }

                CONSTANT_METHOD_TYPE_TAG => {
                    self._cp.push(ConstantMethodTypeInfo(stream.get_u2()?));
                }

                CONSTANT_DYNAMIC_TAG => {
                    self._cp.push(ConstantDynamicInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_INVOKE_DYNAMIC_TAG => {
                    self._cp.push(ConstantInvokeDynamicInfo(stream.get_u2()?, stream.get_u2()?));
                }

                CONSTANT_MODULE_TAG => {
                    self._cp.push(ConstantModuleInfo(stream.get_u2()?));
                }

                CONSTANT_PACKAGE_TAG => {
                    self._cp.push(ConstantPackageInfo(stream.get_u2()?));
                }

                _ => { return Ok(Some(format!("unrecognised tag: {}", tag))) }
            }
        }

        Ok(None)
    }
}
