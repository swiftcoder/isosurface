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
    index_cache::MortonKey,
    marching_cubes_impl::{classify_corners, find_edge_crossings, march_cube},
    math::Vec3,
    mesh::MeshTopologyBuilder,
    sampler::Sample,
    source::ScalarSource,
    traversal::ImplicitOctree,
};

/// Convert isosurfaces to meshes using marching cubes over a linear hashed
/// octree.
///
/// This is a loose implementation of the paper [Fast Generation of Pointerless Octree Duals](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1467-8659.2010.01775.x).
///
/// Pros:
///
/// * Faster than standard marching cubes.
/// * Accurately reproduces grid-aligned sharp edges in the underlying
///   isosurface.
///
/// Cons:
///
/// * Still can't accurately reproduce sharp edges which are not grid-aligned.
pub struct LinearHashedMarchingCubes {
    max_depth: usize,
}

impl LinearHashedMarchingCubes {
    /// Create a new LinearHashedMarchingCubes.
    ///
    /// The depth of the internal octree will be at most `max_depth`, causing
    /// the tree to span the equivalent of a cubic grid at most
    /// `2.pow(max_depth)` in either direction. Distances will be evaluated
    /// in Euclidean space.
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
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
        S: Sample<Signed> + ScalarSource,
        E: Extractor,
    {
        let mut implicit_octree = ImplicitOctree::new(self.max_depth);
        let mut mesh_builder = MeshTopologyBuilder::new(extractor);

        implicit_octree.traverse(source, |keys, corners, values| {
            let cube_index = classify_corners(&values);

            let mut vertices = [Vec3::zero(); 12];
            find_edge_crossings(cube_index, &corners, &values, &mut vertices);
            march_cube(cube_index, |a, b, c| {
                let a = mesh_builder.add_vertex(Some(MortonKey::new(&keys, a)), vertices[a]);
                let b = mesh_builder.add_vertex(Some(MortonKey::new(&keys, b)), vertices[b]);
                let c = mesh_builder.add_vertex(Some(MortonKey::new(&keys, c)), vertices[c]);
                mesh_builder.add_face(a, b, c);
            });
        });

        mesh_builder.build().extract_indices(extractor);
    }
}
