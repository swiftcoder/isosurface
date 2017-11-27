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

use source::{HermiteSource, Source};
use index_cache::IndexCache;
use marching_cubes_impl::{get_offset, interpolate, march_cube};
use marching_cubes_tables::{CORNERS, EDGE_CONNECTION};
use math::Vec3;

/// Extracts meshes from distance fields using the marching cubes algorithm.
pub struct MarchingCubes {
    size: usize,
    layers: [Vec<f32>; 2],
}

impl MarchingCubes {
    /// Create a new MarchingCubes with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size: usize) -> MarchingCubes {
        MarchingCubes {
            size,
            layers: [vec![0f32; size * size], vec![0f32; size * size]],
        }
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

    fn extract_impl<S, E>(&mut self, source: &S, mut extract: E, indices: &mut Vec<u32>)
    where
        S: Source,
        E: FnMut(Vec3) -> (),
    {
        let size_minus_one = self.size - 1;
        let one_over_size = 1.0 / (size_minus_one as f32);

        // Cache layer zero of distance field values
        for y in 0usize..self.size {
            for x in 0..self.size {
                self.layers[0][y * self.size + x] =
                    source.sample(x as f32 * one_over_size, y as f32 * one_over_size, 0.0);
            }
        }

        let mut corners = [Vec3::zero(); 8];
        let mut values = [0f32; 8];

        let mut index_cache = IndexCache::new(self.size);
        let mut index = 0u32;

        for z in 0..self.size {
            // Cache layer N+1 of isosurface values
            for y in 0..self.size {
                for x in 0..self.size {
                    self.layers[1][y * self.size + x] = source.sample(
                        x as f32 * one_over_size,
                        y as f32 * one_over_size,
                        (z + 1) as f32 * one_over_size,
                    );
                }
            }

            // Extract the calls in the current layer
            for y in 0..size_minus_one {
                for x in 0..size_minus_one {
                    for i in 0..8 {
                        corners[i] = Vec3::new(
                            (x + CORNERS[i][0]) as f32 * one_over_size,
                            (y + CORNERS[i][1]) as f32 * one_over_size,
                            (z + CORNERS[i][2]) as f32 * one_over_size,
                        );
                        values[i] = self.layers[CORNERS[i][2]]
                            [(y + CORNERS[i][1]) * self.size + x + CORNERS[i][0]];
                    }

                    march_cube(&values, |edge: usize| {
                        let cached_index = index_cache.get(x, y, edge);
                        if cached_index > 0 {
                            indices.push(cached_index);
                        } else {
                            let u = EDGE_CONNECTION[edge][0];
                            let v = EDGE_CONNECTION[edge][1];

                            index_cache.put(x, y, edge, index);
                            indices.push(index);
                            index += 1;

                            let offset = get_offset(values[u], values[v]);
                            let vertex = interpolate(corners[u], corners[v], offset);
                            extract(vertex);
                        }
                    });
                    index_cache.advance_cell();
                }
                index_cache.advance_row();
            }
            index_cache.advance_layer();

            self.layers.swap(0, 1);
        }
    }
}
