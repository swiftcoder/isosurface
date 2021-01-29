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
use crate::{
    distance::Distance,
    marching_cubes_tables::{EDGE_CONNECTION, EDGE_CROSSING_MASK, TRIANGLE_CONNECTION},
    math::Vec3,
    sampler::Sample,
    source::HermiteSource,
};

/// Given the signed distances at each corner of the cube, classify them as
/// either inside or outside the surface, and return the bitmask of the result
/// (where a bit is set if the corner is outside the surface, and unset if
/// inside).
pub fn classify_corners<D>(values: &[D; 8]) -> usize
where
    D: Distance,
{
    let mut cube_index = 0;
    for i in 0..8 {
        if !values[i].is_positive() {
            cube_index |= 1 << i;
        }
    }
    cube_index
}

pub fn find_edge_crossings<D>(
    cube_index: usize,
    corners: &[Vec3; 8],
    values: &[D; 8],
    vertices: &mut [Vec3; 12],
) where
    D: Distance,
{
    let edges = EDGE_CROSSING_MASK[cube_index];

    for i in 0..12 {
        if (edges & (1 << i)) != 0 {
            let [u, v] = EDGE_CONNECTION[i];

            vertices[i] = D::find_crossing_point(values[u], values[v], corners[u], corners[v]);
        }
    }
}

pub fn sample_normals_at_corners<D, S>(source: &S, corners: &[Vec3; 8], normals: &mut [Vec3; 8])
where
    D: Distance,
    S: Sample<D> + HermiteSource,
{
    for i in 0..8 {
        normals[i] = source
            .sample_normal(corners[i])
            .normalised()
            .unwrap_or_default();
    }
}

pub fn sample_normals_at_edge_crossings<D, S>(
    cube_index: usize,
    source: &S,
    vertices: &[Vec3; 12],
    normals: &mut [Vec3; 12],
) where
    D: Distance,
    S: Sample<D> + HermiteSource,
{
    let edges = EDGE_CROSSING_MASK[cube_index];

    for i in 0..12 {
        if (edges & (1 << i)) != 0 {
            normals[i] = source
                .sample_normal(vertices[i])
                .normalised()
                .unwrap_or_default();
        }
    }
}

/// March a single cube, given the 8 corner vertices, and the density at each
/// vertex.
///
/// The `edge_func` will be invoked once for each vertex in the resulting mesh
/// data, with the index of the edge on which the vertex falls. Each triplet of
/// invocations forms one triangle.
///
/// It would in many ways be simple to output triangles directly, but callers
/// needing to produce indexed geometry will want to deduplicate vertices before
/// forming triangles.
pub fn march_cube<F>(cube_index: usize, mut face_callback: F)
where
    F: FnMut(usize, usize, usize) -> (),
{
    for i in 0..5 {
        if TRIANGLE_CONNECTION[cube_index][3 * i] < 0 {
            break;
        }

        face_callback(
            TRIANGLE_CONNECTION[cube_index][3 * i + 0] as usize,
            TRIANGLE_CONNECTION[cube_index][3 * i + 1] as usize,
            TRIANGLE_CONNECTION[cube_index][3 * i + 2] as usize,
        );
    }
}
