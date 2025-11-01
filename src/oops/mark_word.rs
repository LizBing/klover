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

use modular_bitfield::{bitfield, prelude::{B1, B2, B26, B31, B4}};

#[bitfield(bits = 64)]
#[derive(Clone, Copy)]
struct MarkWord {
    _lock: B2,
    _biased: B1,
    _age: B4,
    _hash: B31,
    _klass_ptr: B26
}
