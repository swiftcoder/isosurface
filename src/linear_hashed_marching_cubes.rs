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

use crate::linear_hashed_octree::LinearHashedOctree;
use crate::marching_cubes_impl::{get_offset, interpolate, march_cube};
use crate::marching_cubes_tables::EDGE_CONNECTION;
use crate::math::Vec3;
use crate::morton::Morton;
use crate::source::{HermiteSource, Norm, Source};
use std::collections::HashMap;

// Morton cube corners are ordered differently to the marching cubes tables, so remap them to match.
const REMAP_CUBE: [usize; 8] = [2, 3, 1, 0, 6, 7, 5, 4];

// Used to compute the diagonal dimension (i.e. 3-dimensional hypotenuse) of a cube.
const SQRT_OF_3: f32 = 1.732_050_807_57;

// Uniquely identifies an edge by its terminal vertices
#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Edge(Morton, Morton);

impl Edge {
    fn new(u: Morton, v: Morton) -> Self {
        if u > v {
            Edge(v, u)
        } else {
            Edge(u, v)
        }
    }
}

/// Extracts meshes from distance fields using marching cubes over a linear hashed octree.
pub struct LinearHashedMarchingCubes {
    max_depth: usize,
    norm: Norm,
}

impl LinearHashedMarchingCubes {
    /// Create a new LinearHashedMarchingCubes.
    ///
    /// The depth of the internal octree will be at most `max_depth`, causing the tree to span the
    /// equivalent of a cubic grid at most `2.pow(max_depth)` in either direction. Distances
    /// will be evaluated in Euclidean space.
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            norm: Norm::Euclidean,
        }
    }

    /// Create a new LinearHashedMarchingCubes.
    ///
    /// The depth of the internal octree will be at most `max_depth`, causing the tree to span the
    /// equivalent of a cubic grid at most `2.pow(max_depth)` in either direction. Distances will
    /// be evaluated in accordance with the provided Norm.
    pub fn with_norm(max_depth: usize, norm: Norm) -> Self {
        Self { max_depth, norm }
    }

    /// Extracts a mesh from the given [`Source`](../source/trait.Source.html).
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the number of steps
    /// determined by the size provided to the constructor.
    ///
    /// Extracted vertices will be appended to `vertices` as triples of (x, y, z)
    /// coordinates. Extracted triangles will be appended to `indices` as triples of
    /// vertex indices.
    pub fn extract<S>(&mut self, source: &S, vertices: &mut Vec<f32>, indices: &mut Vec<u32>)
    where
        S: Source,
    {
        self.extract_impl(
            source,
            |v: Vec3| {
                vertices.push(v.x);
                vertices.push(v.y);
                vertices.push(v.z);
            },
            indices,
        );
    }

    /// Extracts a mesh from the given [`HermiteSource`](../source/trait.HermiteSource.html).
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the number of steps
    /// determined by the size provided to the constructor.
    ///
    /// Extracted vertices will be appended to `vertices` as triples of (x, y, z)
    /// coordinates, followed by the surface normals as triples of (x, y, z) dimensions. Extracted
    /// triangles will be appended to `indices` as triples of vertex indices.
    pub fn extract_with_normals<S>(
        &mut self,
        source: &S,
        vertices: &mut Vec<f32>,
        indices: &mut Vec<u32>,
    ) where
        S: HermiteSource,
    {
        self.extract_impl(
            source,
            |v: Vec3| {
                let n = source.sample_normal(v.x, v.y, v.z);
                vertices.push(v.x);
                vertices.push(v.y);
                vertices.push(v.z);
                vertices.push(n.x);
                vertices.push(n.y);
                vertices.push(n.z);
            },
            indices,
        );
    }

    fn extract_impl<S, E>(&mut self, source: &S, extract: E, indices: &mut Vec<u32>)
    where
        S: Source,
        E: FnMut(Vec3) -> (),
    {
        let octree = self.build_octree(source);
        let primal_vertices = self.compute_primal_vertices(&octree);
        let mut base_index = 0;
        self.extract_surface(&octree, &primal_vertices, indices, &mut base_index, extract);
    }

    fn diagonal(&self, distance: f32) -> f32 {
        match self.norm {
            Norm::Euclidean => distance * SQRT_OF_3,
            Norm::Max => distance,
        }
    }

    fn build_octree<S>(&mut self, source: &S) -> LinearHashedOctree<f32>
    where
        S: Source,
    {
        let max_depth = self.max_depth;
        let mut octree = LinearHashedOctree::new();

        octree.build(
            |key: Morton, distance: &f32| {
                let level = key.level();
                let size = key.size();
                level < 2 || (level < max_depth && distance.abs() <= self.diagonal(size))
            },
            |key: Morton| {
                let p = key.center();
                source.sample(p.x, p.y, p.z)
            },
        );

        octree
    }

    fn compute_primal_vertices(
        &mut self,
        octree: &LinearHashedOctree<f32>,
    ) -> HashMap<Morton, usize> {
        let mut primal_vertices = HashMap::new();

        octree.walk_leaves(|key: Morton| {
            let level = key.level();
            for i in 0..8 {
                let vertex = key.primal_vertex(level, i);

                if vertex != Morton::with_key(0) {
                    if let Some(&existing_level) = primal_vertices.get(&vertex) {
                        if level > existing_level {
                            primal_vertices.insert(vertex, level);
                        }
                    } else {
                        primal_vertices.insert(vertex, level);
                    }
                }
            }
        });

        primal_vertices
    }

    fn extract_surface<E>(
        &mut self,
        octree: &LinearHashedOctree<f32>,
        primal_vertices: &HashMap<Morton, usize>,
        indices: &mut Vec<u32>,
        base_index: &mut u32,
        mut extract: E,
    ) where
        E: FnMut(Vec3) -> (),
    {
        let mut index_map = HashMap::new();

        let mut duals = [Morton::new(); 8];
        let mut dual_distances = [0.0; 8];

        for (key, &level) in primal_vertices {
            for i in 0..8 {
                let mut m = key.dual_vertex(level, i);
                while m > Morton::new() {
                    if let Some(&distance) = octree.get_node(&m) {
                        duals[i] = m;
                        dual_distances[i] = distance;
                        break;
                    }
                    m = m.parent();
                }
            }

            self.march_one_cube(
                duals,
                dual_distances,
                &mut index_map,
                indices,
                base_index,
                &mut extract,
            );
        }
    }

    fn march_one_cube<E>(
        &mut self,
        nodes: [Morton; 8],
        dual_distances: [f32; 8],
        index_map: &mut HashMap<Edge, u32>,
        indices: &mut Vec<u32>,
        base_index: &mut u32,
        extract: &mut E,
    ) where
        E: FnMut(Vec3) -> (),
    {
        let mut reordered_nodes = [Morton::with_key(0); 8];
        let mut corners = [Vec3::zero(); 8];
        let mut values = [0f32; 8];

        for i in 0..8 {
            let key = nodes[REMAP_CUBE[i]];
            let distance = dual_distances[REMAP_CUBE[i]];

            reordered_nodes[i] = key;
            corners[i] = key.center();
            values[i] = distance;
        }

        march_cube(&values, |edge: usize| {
            let u = EDGE_CONNECTION[edge][0];
            let v = EDGE_CONNECTION[edge][1];

            let edge_key = Edge::new(reordered_nodes[u], reordered_nodes[v]);

            if let Some(&index) = index_map.get(&edge_key) {
                indices.push(index);
            } else {
                let index = *base_index;
                *base_index += 1;

                index_map.insert(edge_key, index);
                indices.push(index);

                let offset = get_offset(values[u], values[v]);
                let vertex = interpolate(corners[u], corners[v], offset);
                extract(vertex);
            }
        });
    }
}
