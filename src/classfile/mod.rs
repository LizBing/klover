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

mod access_flags;
mod class_loader;
mod constant_pool;
mod info_types;
mod parser;
mod stream;

const MAGIC:                u32 = 0xCAFEBABE;
const SMALLEST_MAJOR:       u16 =         45;
const LARGEST_MAJOR:        u16 =         65;
const GENERAL_MAJOR:        u16 =         56;
const SPECIFIC_MINOR_0:     u16 =          0;
const SPECIFIC_MINOR_65535: u16 =      65535;
