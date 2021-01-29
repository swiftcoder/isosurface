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
    distance::Signed,
    extractor::Extractor,
    feature::PlaceFeatureInCell,
    index_cache::GridKey,
    marching_cubes_impl::{
        classify_corners, find_edge_crossings, march_cube, sample_normals_at_corners,
    },
    math::Vec3,
    mesh::MeshTopologyBuilder,
    sampler::Sample,
    source::HermiteSource,
    traversal::DualGrid,
};

#[cfg(doc)]
use crate::feature::{MinimiseQEF, ParticleBasedMinimisation};

/// Convert isosurfaces to meshes using dual contouring.
///
/// If you pass [MinimiseQEF] to the constructor this implements the classic [Dual Contouring of Hermite Data](https://doi.org/10.1145/566570.566586). If you instead pass in [ParticleBasedMinimisation] this becomes the improved version of dual contouring from [Efficient and Quality Contouring Algorithms on the GPU](https://doi.org/10.1111/j.1467-8659.2010.01825.x).
///
/// Pros:
/// * Decent reproduction of sharp edges even when not grid-aligned.
///
/// Cons:
/// * Feature placement can be very sensitive to the quality of input data.
pub struct DualContouring<P: PlaceFeatureInCell> {
    dual_grid: DualGrid<Signed>,
    place_feature: P,
}

impl<P: PlaceFeatureInCell> DualContouring<P> {
    /// Create a new DualContouring with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size: usize, place_feature: P) -> Self {
        Self {
            dual_grid: DualGrid::new(size),
            place_feature,
        }
    }

    /// Extracts a mesh from the given [Sample].
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the
    /// number of steps determined by the size provided to the constructor.
    ///
    /// The resulting vertex and face data will be returned via the provided
    /// Extractor.
    pub fn extract<S, E>(&mut self, source: &S, extractor: &mut E)
    where
        S: Sample<Signed> + HermiteSource,
        E: Extractor,
    {
        let mut mesh_builder = MeshTopologyBuilder::new(extractor);
        let mut normals = [Vec3::zero(); 8];

        let dual_grid = &mut self.dual_grid;
        let place_feature = &mut self.place_feature;

        dual_grid.traverse(
            source,
            Some(|corners: &[Vec3; 8], values: &[Signed; 8]| {
                let cube_index = classify_corners(&values);
                if cube_index == 0 || cube_index == 255 {
                    return None;
                }

                sample_normals_at_corners(source, &corners, &mut normals);

                Some(place_feature.place_feature_in_cell(corners, &normals))
            }),
            |keys, corners, values| {
                let cube_index = classify_corners(&values);

                let mut vertices = [Vec3::zero(); 12];
                find_edge_crossings(cube_index, &corners, &values, &mut vertices);

                march_cube(cube_index, |a, b, c| {
                    let a = mesh_builder.add_vertex(Some(GridKey::new(keys, a)), vertices[a]);
                    let b = mesh_builder.add_vertex(Some(GridKey::new(keys, b)), vertices[b]);
                    let c = mesh_builder.add_vertex(Some(GridKey::new(keys, c)), vertices[c]);

                    mesh_builder.add_face(a, b, c);
                });
            },
        );

        mesh_builder.build().extract_indices(extractor);
    }
}
