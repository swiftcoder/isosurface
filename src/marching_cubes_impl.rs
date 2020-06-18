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

use crate::marching_cubes_tables::TRIANGLE_CONNECTION;
use std::ops::{Add, Mul};

/// March a single cube, given the 8 corner vertices, and the density at each vertex.
///
/// The `edge_func` will be invoked once for each vertex in the resulting mesh data, with the index
/// of the edge on which the vertex falls. Each triplet of invocations forms one triangle.
///
/// It would in many ways be simple to output triangles directly, but callers needing to produce
/// indexed geometry will want to deduplicate vertices before forming triangles.
pub fn march_cube<E>(values: &[f32; 8], mut edge_func: E)
where
    E: FnMut(usize) -> (),
{
    let mut cube_index = 0;
    for i in 0..8 {
        if values[i] <= 0.0 {
            cube_index |= 1 << i;
        }
    }

    for i in 0..5 {
        if TRIANGLE_CONNECTION[cube_index][3 * i] < 0 {
            break;
        }

        for j in 0..3 {
            let edge = TRIANGLE_CONNECTION[cube_index][3 * i + j] as usize;

            edge_func(edge);
        }
    }
}

/// Calculate the position of the vertex along an edge, given the density at either end of the edge.
pub fn get_offset(a: f32, b: f32) -> f32 {
    let delta = b - a;
    if delta == 0.0 {
        0.5
    } else {
        -a / delta
    }
}

/// Linearly Interpolate between two floating point values
pub fn interpolate<T>(a: T, b: T, t: f32) -> T
where
    T: Add<T, Output = T> + Mul<f32, Output = T>,
{
    a * (1.0 - t) + b * t
}
