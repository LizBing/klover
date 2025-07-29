/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use phf::phf_map;

use crate::runtime::flag_sys::vm_flag::{VMFlag, VMFlagData};

pub static INTP_STACK_SIZE: VMFlagData<usize> = VMFlagData::new(
    "IntpStackSize",
    4,
    "Size of interpreter stack(MB).", 
    None);

static VM_FLAG_MAP: phf::Map<&'static str, VMFlag> = phf_map! {
    "IntpStackSize" => VMFlag::USIZE_FLAG(&INTP_STACK_SIZE)
};
