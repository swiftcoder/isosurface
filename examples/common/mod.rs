// Copyright 2017 Tristam MacDonald
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

pub mod sources;

use std::slice;

/// This is used to reinterpret slices of floats as slices of repr(C) structs, without any
/// copying. It is optimal, but it is also punching holes in the type system. I hope that Rust
/// provides safe functionality to handle this in the future. In the meantime, reproduce
/// this workaround at your own risk.
pub fn reinterpret_cast_slice<S, T>(input : &[S], length : usize) -> &[T] {
    unsafe {
        slice::from_raw_parts(input.as_ptr() as *const T, length)
    }
}
