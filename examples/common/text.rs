// Copyright 2018 Tristam MacDonald
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

//! Conveniences to make the `glium_text` API easier to use in samples.

use cgmath::{Matrix4, Vector3};

/// Produce a transform matrix that will display text at offset column `x`, row `y`, in a
/// display-filling coordinate space N characters wide and N*aspect rows high.
pub fn layout_text(characters_per_row: f32, aspect: f32, x: f32, y: f32) -> Matrix4<f32> {
    let inv_scale = 2.0 / characters_per_row;
    Matrix4::from_translation(Vector3::new(-1.0, -1.0, 0.0))
        * Matrix4::from_nonuniform_scale(inv_scale, inv_scale * aspect, 1.0)
        * Matrix4::from_translation(Vector3::new(x, y, 0.0))
}
