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

use cafebabe::attributes::CodeData;

use crate::gc::oop_map::OopMap;

pub struct Method<'a> {
    _oop_map: OopMap,
    _code_data: Option<&'a CodeData<'a>>
}


impl<'a> Method<'a> {
    pub fn oop_map(&self) -> &OopMap {
        &self._oop_map
    }

    pub fn code_data(&self) -> Option<&CodeData<'a>> {
        self._code_data
    }
}

