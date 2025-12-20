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

use std::ops::Neg;

use crate::utils::global_defs::{JDouble, JFloat};

pub trait FloatType: Copy + PartialOrd + Neg<Output = Self> {
    const NAN: Self;

    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;

    fn is_nan(self) -> bool;
}

impl FloatType for JFloat {
    const NAN: Self = f32::NAN;

    const ZERO: Self = 0.0f32;
    const ONE: Self = 1.0f32;
    const TWO: Self = 2.0f32;

    #[inline]
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}

impl FloatType for JDouble {
    const NAN: Self = f64::NAN;

    const ZERO: Self = 0.0f64;
    const ONE: Self = 1.0f64;
    const TWO: Self = 2.0f64;

    #[inline]
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}
