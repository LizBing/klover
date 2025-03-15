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

use std::borrow::Cow;

use cafebabe::ParseError;

mod class_linker;
// pub mod rtcp;

pub struct ClassLoader<'a> {
    _class_file: cafebabe::ClassFile<'a>,
}

pub enum CLError {
    Parser(ParseError),
    Linker(String)
}

impl<'a> ClassLoader<'a> {
    pub fn symbolic_ref(&self) -> &Cow<str> {
        &self._class_file.this_class
    }

    pub fn with_path(path: &'a str) -> Result<Self, CLError> {
        let mut cl = ClassLoader {
            _class_file: match cafebabe::parse_class(path.as_bytes()) {
                Ok(res) => res,
                Err(e) => {
                    return Err(CLError::Parser(e));
                }
            },
        };

        Ok(cl)
    }
}

impl<'a> ClassLoader<'a> {
}
