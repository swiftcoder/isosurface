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
    distance::Distance, linear_hashed_octree::LinearHashedOctree,
    marching_cubes_tables::REMAP_CUBE, math::Vec3, morton::Morton, sampler::Sample,
};
use std::collections::HashMap;

/// Traverses over the leaves in a sparse octree that uses morton coordinates to
/// represent nodes in the tree.
pub struct ImplicitOctree {
    max_depth: usize,
}

impl ImplicitOctree {
    /// Create a implicit octree with depth N, which is equivalent to a cubic
    /// grid with dimensions 2^N along each axis.
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Build an implicit octree by sampling from the provided Sampler to find
    /// the surface crossings. Then place a vertex at the center of each
    /// leaf node, and traverse the leaf nodes, invoking the callback for
    /// each 2x2x2 cube of neighbouring leaf vertices. The callback will be
    /// provided the Morton coordinates for each vertex, the vertices
    /// themselves, and the field values at those vertices.
    pub fn traverse<D, S, C>(&mut self, source: &S, mut callback: C)
    where
        D: Distance,
        S: Sample<D>,
        C: FnMut(&[Morton; 8], &[Vec3; 8], &[D; 8]),
    {
        let mut octree = LinearHashedOctree::new();

        octree.build(
            |key: Morton, distance: &D| {
                let level = key.level();
                let size = key.size();
                // TODO: figure out how to construct an octree over a directed distance field
                level < 2 || (level < self.max_depth && distance.within_extent(size))
            },
            |key: Morton| {
                let p = key.center();
                source.sample(p)
            },
        );

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

        let mut keys = [Morton::new(); 8];
        let mut corners = [Vec3::zero(); 8];
        let mut values = [D::zero(); 8];

        for (key, level) in primal_vertices {
            for i in 0..8 {
                let mut m = key.dual_vertex(level, REMAP_CUBE[i]);
                while m > Morton::new() {
                    if let Some(&distance) = octree.get_node(&m) {
                        keys[i] = m;
                        corners[i] = m.center();
                        values[i] = distance;
                        break;
                    }
                    m = m.parent();
                }
            }

            callback(&keys, &corners, &values);
        }
    }
}
