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

const CONSTANT_CLASS_TAG: u8 = 7;
const CONSTANT_FIELDREF_TAG: u8 = 9;
const CONSTANT_METHODREF_TAG: u8 = 10;
const CONSTANT_INTERFACE_METHODREF_TAG: u8 = 11;
const CONSTANT_STRING_TAG: u8 = 8;
const CONSTANT_INTEGER_TAG: u8 = 3;
const CONSTANT_FLOAT_TAG: u8 = 4;
const CONSTANT_LONG_TAG: u8 = 5;
const CONSTANT_DOUBLE_TAG: u8 = 6;
const CONSTANT_NAME_AND_TYPE_TAG: u8 = 12;
const CONSTANT_UTF8_TAG: u8 = 1;
const CONSTANT_METHOD_HANDLE_TAG: u8 = 15;
const CONSTANT_METHOD_TYPE_TAG: u8 = 16;
const CONSTANT_DYNAMIC_TAG: u8 = 17;
const CONSTANT_INVOKE_DYNAMIC_TAG: u8 = 18;
const CONSTANT_MODULE_TAG: u8 = 19;
const CONSTANT_PACKAGE_TAG: u8 = 20;


pub enum ConstantPoolEntry {
    // (name index)
    ConstantClassInfo(u16),

    // (class index, name and type index)
    ConstantFieldrefInfo(u16, u16),
    ConstantMethodrefInfo(u16, u16),
    ConstantInterfaceMethodrefInfo(u16, u16),

    // (string index)
    ConstantStringInfo(u16),

    // (data)
    ConstantIntegerInfo(u32),
    ConstantFloatInfo(f32),

    ConstantLongInfo(u64),
    ConstantDoubleInfo(f64),

    // (name index, descriptor index)
    ConstantNameAndTypeInfo(u16, u16),

    ConstantUTF8Info(String),

    // [reference kind(1~9), reference index]
    ConstantMethodHandleInfo(u8, u16),

    // (descriptor index)
    ConstantMethodTypeInfo(u16),

    // (bootstrap method attr index, name and type index)
    ConstantDynamicInfo(u16, u16),
    ConstantInvokeDynamicInfo(u16, u16),

    // (name index)
    ConstantModuleInfo(u16),
    ConstantPackageInfo(u16),
}

pub struct ConstantPool {
    _array: Vec<ConstantPoolEntry>
}
