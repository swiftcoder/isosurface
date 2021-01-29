// Copyright 2021 Tristam MacDonald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub mod svd;
pub mod vector;

pub use vector::*;

use std::ops::{Add, Mul};

pub fn lerp<T>(a: T, b: T, f: f32) -> T
where
    f32: Mul<T, Output = T>,
    T: Add<Output = T>,
{
    (1.0 - f) * a + f * b
}
