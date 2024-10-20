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

use super::stream::ClassFileStream;

pub trait InfoType: Sized {
    fn extract(stream: &mut ClassFileStream) -> io::Result<Self>;

    fn extract_vec(stream: &mut ClassFileStream, count: usize)
        -> io::Result<Vec<Self>>
    {
        let mut res = Vec::new();
        res.reserve_exact(count);

        for _ in 0..count {
            res.push(Self::extract(stream)?);
        }

        Ok(res)
    }
}

pub struct FieldInfo {
    _access_flags: u16,
    _name_index: u16,
    _desc_index: u16,
    _attrs: Vec<AttributeInfo>
}

impl InfoType for FieldInfo {
    fn extract(stream: &mut ClassFileStream) -> io::Result<Self> {
        let mut res = FieldInfo {
            _access_flags: stream.get_u2()?,
            _name_index: stream.get_u2()?,
            _desc_index: stream.get_u2()?,
            _attrs: Vec::new()
        };

        let attrs_count = stream.get_u2()? as usize;
        res._attrs = AttributeInfo::extract_vec(stream, attrs_count)?;

        Ok(res)
    }
}

pub struct MethodInfo {
    _access_flags: u16,
    _name_index: u16,
    _desc_index: u16,
    _attrs: Vec<AttributeInfo>
}

impl InfoType for MethodInfo {
    fn extract(stream: &mut ClassFileStream) -> io::Result<Self> {
        let mut res = MethodInfo {
            _access_flags: stream.get_u2()?,
            _name_index: stream.get_u2()?,
            _desc_index: stream.get_u2()?,
            _attrs: Vec::new()
        };

        let attrs_count = stream.get_u2()? as usize;
        res._attrs = AttributeInfo::extract_vec(stream, attrs_count)?;

        Ok(res)
    }
}

pub struct AttributeInfo {}

impl InfoType for AttributeInfo {
    fn extract(stream: &mut ClassFileStream) -> io::Result<Self> {}
}
