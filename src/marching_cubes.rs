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
    extractor::Extractor,
    index_cache::GridKey,
    marching_cubes_impl::{classify_corners, find_edge_crossings, march_cube},
    math::Vec3,
    mesh::MeshTopologyBuilder,
    sampler::Sample,
    traversal::PrimalGrid,
};

/// Convert isosurfaces to meshes using marching cubes.
///
/// This is the classical isosurface extraction algorithm from [Marching cubes: A high resolution 3D surface construction algorithm](https://doi.org/10.1145/37402.37422).
///
/// Pros:
///
/// * Pretty fast.
/// * The classics are timeless.
///
/// Cons:
///
/// * Produces a lot of small triangle slivers.
/// * Can't accurately reproduce sharp edges in the isosurface.
pub struct MarchingCubes<D: Distance> {
    primal_grid: PrimalGrid<D>,
}

impl<D: Distance> MarchingCubes<D> {
    /// Create a new MarchingCubes with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size: usize) -> Self {
        Self {
            primal_grid: PrimalGrid::new(size),
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
        S: Sample<D>,
        E: Extractor,
    {
        let mut mesh_builder = MeshTopologyBuilder::new(extractor);

        self.primal_grid.traverse(source, |keys, corners, values| {
            let cube_index = classify_corners(&values);

            let mut vertices = [Vec3::zero(); 12];
            find_edge_crossings(cube_index, &corners, &values, &mut vertices);

            march_cube(cube_index, |a, b, c| {
                let a = mesh_builder.add_vertex(Some(GridKey::new(keys, a)), vertices[a]);
                let b = mesh_builder.add_vertex(Some(GridKey::new(keys, b)), vertices[b]);
                let c = mesh_builder.add_vertex(Some(GridKey::new(keys, c)), vertices[c]);

                mesh_builder.add_face(a, b, c);
            });
        });

        mesh_builder.build().extract_indices(extractor);
    }
}
