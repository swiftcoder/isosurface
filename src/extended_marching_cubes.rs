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
    distance::Directed,
    extractor::Extractor,
    feature::{LocalTopology, MinimiseQEF, TangentPlanes},
    index_cache::GridKey,
    marching_cubes_impl::{
        classify_corners, find_edge_crossings, march_cube, sample_normals_at_edge_crossings,
    },
    marching_cubes_tables::EDGE_LOOPS,
    math::Vec3,
    mesh::{MeshTopology, MeshTopologyBuilder, VertexHandle},
    sampler::Sample,
    source::HermiteSource,
    traversal::PrimalGrid,
};
use std::collections::HashSet;

/// Convert isosurfaces to meshes using extended marching cubes.
///
/// This is an implementation of the paper [Feature Sensitive Surface Extraction from Volume Data](https://dl.acm.org/doi/abs/10.1145/383259.383265).
/// Pros:
///
/// * Decent reproduction of sharp edges even when not grid-aligned.
///
/// Cons:
///
/// * May produce even more small triangle slivers on sharp edges.
/// * Extraction has dependencies on neighbouring chunks at the final
///   edge-flipping step.
pub struct ExtendedMarchingCubes {
    primal_grid: PrimalGrid<Directed>,
}

impl ExtendedMarchingCubes {
    /// Create a new ExtendedMarchingCubes with the given chunk size.
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
        S: Sample<Directed> + HermiteSource,
        E: Extractor,
    {
        let mut mesh_builder = MeshTopologyBuilder::new(extractor);
        let mut features = HashSet::new();

        self.primal_grid.traverse(source, |keys, corners, values| {
            let mut vertices = [Vec3::zero(); 12];
            let mut normals = [Vec3::zero(); 12];

            let cube_index = classify_corners(&values);
            find_edge_crossings(cube_index, &corners, &values, &mut vertices);
            sample_normals_at_edge_crossings(cube_index, source, &vertices, &mut normals);

            Self::march_cube_extended(
                &mut features,
                &mut mesh_builder,
                cube_index,
                vertices,
                normals,
                keys,
            );
        });

        let mut mesh = mesh_builder.build();
        Self::flip_feature_edges(features, &mut mesh);
        mesh.extract_indices(extractor);
    }

    fn march_cube_extended<E>(
        features: &mut HashSet<VertexHandle>,
        mesh_builder: &mut MeshTopologyBuilder<GridKey, E>,
        cube_index: usize,
        vertices: [Vec3; 12],
        normals: [Vec3; 12],
        keys: &[(usize, usize, usize); 8],
    ) where
        E: Extractor,
    {
        let tangent_planes = TangentPlanes::from_edge_crossings(cube_index, &vertices, &normals);

        if let LocalTopology::Planar = tangent_planes.feature {
            // No feature detected, so we can just use traditional marching cubes
            march_cube(cube_index, |a, b, c| {
                let a = mesh_builder.add_vertex(Some(GridKey::new(keys, a)), vertices[a]);
                let b = mesh_builder.add_vertex(Some(GridKey::new(keys, b)), vertices[b]);
                let c = mesh_builder.add_vertex(Some(GridKey::new(keys, c)), vertices[c]);

                mesh_builder.add_face(a, b, c);
            });
        } else {
            // We have a feature, so we need to spawn a new vertex close to the feature

            // Position the feature point by minimising the error in the system of equations
            // formed by the tangent planes. Due to numerical inaccuracies the result may
            // diverge a little from the expected location.
            let feature_point = MinimiseQEF::place_feature_with_tangents(&tangent_planes);

            // Spawn a new vertex at the feature point
            let center_index = mesh_builder.add_vertex(None, feature_point);

            // extract_vertex(feature_point);
            // let center_index = work.mesh.add_vertex();

            // Flag the vertex as a feature, since we'll need to flip some edges around it
            // later
            features.insert(center_index);

            // Form a triangle fan for each edge loop in the marching cubes triangle
            // configuration
            let loop_count = EDGE_LOOPS[cube_index][0] as usize;
            let mut offset = 1 + loop_count;
            for loops in 0..loop_count {
                let vertex_count = EDGE_LOOPS[cube_index][1 + loops] as usize;

                for i in 0..vertex_count {
                    let j = (i + 1) % vertex_count;

                    let e0 = EDGE_LOOPS[cube_index][offset + i] as usize;
                    let a = mesh_builder.add_vertex(Some(GridKey::new(keys, e0)), vertices[e0]);

                    let e1 = EDGE_LOOPS[cube_index][offset + j] as usize;
                    let b = mesh_builder.add_vertex(Some(GridKey::new(keys, e1)), vertices[e1]);

                    mesh_builder.add_face(a, b, center_index);
                }
                offset += vertex_count;
            }
        }
    }

    fn flip_feature_edges(features: HashSet<VertexHandle>, mesh: &mut MeshTopology) {
        // We can't modify the mesh while iterating it (it would anger the borrow
        // checker), so accumulate edges that need flipping and process them
        // afterwards
        let mut flips = HashSet::new();

        for &edge in mesh.edges() {
            // If this edge already joins two features, don't flip it
            if features.contains(&edge.start()) && features.contains(&edge.end()) {
                continue;
            }

            let faces = mesh.adjoining_faces(edge);

            // Only try to flip edges that adjoin exactly two faces
            if let [face_a, face_b] = faces[..] {
                // If flipping this edge will connect two features, add it to the list
                if features.contains(&face_a.vertex_opposite(edge))
                    && features.contains(&face_b.vertex_opposite(edge))
                {
                    flips.insert(edge);
                }
            }
        }

        // Now we can flip all the edges we found earlier
        for edge in flips {
            mesh.rotate_edge(edge);
        }
    }
}
