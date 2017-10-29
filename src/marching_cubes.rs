// Copyright 2017 Tristam MacDonald
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

use index_cache::IndexCache;
use marching_cubes_tables::{CORNERS, EDGE_CONNECTION, TRIANGLE_CONNECTION};

/// A source capable of sampling a signed distance field at discrete coordinates.
pub trait Source {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return the signed distance (i.e. negative for coodinates inside the surface),
    /// as our Marching Cubes implementation will evaluate the surface at the zero-crossing.
    fn sample(&self, x : f32, y : f32, z : f32) -> f32;
}

/// Extracts meshes from distance fields using the marching cubes algorithm.
pub struct MarchingCubes {
    size : usize,
    layers : [Vec<f32>; 2]
}

fn get_offset(a : f32, b : f32) -> f32 {
    let delta = b - a;
    if delta == 0.0 {0.5} else {-a/delta}
}

fn interpolate(a : f32, b : f32, t : f32) -> f32 {
    a * (1.0 - t) + b * t
}

impl MarchingCubes {

    /// Create a new MarchingCubes with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size : usize) -> MarchingCubes {
        MarchingCubes {
            size,
            layers: [vec![0f32; size*size], vec![0f32; size*size]],
        }
    }

    /// Extracts a mesh from the given [`Source`](trait.Source.html).
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the number of steps
    /// determined by the size provided to the constructor.
    ///
    /// Extracted vertices will be appended to `vertices` as triples of (x, y, z)
    /// coordinates. Extracted triangles will be appended to `indices` as triples of
    /// vertex indices.
    pub fn extract<S>(&mut self, source : &S, vertices : &mut Vec<f32>, indices : &mut Vec<u32>)
        where S : Source {

        let size_minus_one = self.size - 1;
        let one_over_size = 1.0 / (size_minus_one as f32);

        // Cache layer zero of distance field values
        for y in 0usize..self.size {
            for x in 0..self.size {
                self.layers[0][y*self.size + x] = source.sample(x as f32 * one_over_size,
                                                                y as f32 * one_over_size,
                                                                0.0);
            }
        }

        let mut corners = [[0f32; 3]; 8];
        let mut values = [0f32; 8];

        let mut index_cache = IndexCache::new(self.size);

        for z in 0..self.size {

            // Cache layer N+1 of isosurface values
            for y in 0..self.size {
                for x in 0..self.size {
                    self.layers[1][y*self.size + x] = source.sample(x as f32 * one_over_size,
                                                                    y as f32 * one_over_size,
                                                                    (z+1) as f32 * one_over_size);
                }
            }

            // Extract the calls in the current layer
            for y in 0..size_minus_one {
                for x in 0..size_minus_one {
                    for i in 0..8 {
                        corners[i] = [
                            (x + CORNERS[i][0]) as f32 * one_over_size,
                            (y + CORNERS[i][1]) as f32 * one_over_size,
                            (z + CORNERS[i][2]) as f32 * one_over_size
                        ];
                        values[i] = self.layers[CORNERS[i][2]][(y + CORNERS[i][1]) * self.size + x + CORNERS[i][0]];
                    }

                    let mut cube_index = 0;
                    for i in 0usize..8 {
                        if values[i] <= 0.0 {
                            cube_index |= 1 << i;
                        }
                    }

                    for i in 0..5 {
                        if TRIANGLE_CONNECTION[cube_index][3*i] < 0 {
                            break;
                        }

                        for j in 0..3 {
                            let vert = TRIANGLE_CONNECTION[cube_index][3 * i + j] as usize;
                            let cached_index = index_cache.get(x, y, vert);
                            if  cached_index > 0 {
                                indices.push(cached_index);
                            } else {
                                let u = EDGE_CONNECTION[vert][0];
                                let v = EDGE_CONNECTION[vert][1];

                                let index = (vertices.len() / 3) as u32;
                                index_cache.put(x, y, vert, index);
                                indices.push(index);

                                let offset = get_offset(values[u], values[v]);
                                for k in 0..3 {
                                    vertices.push(interpolate(corners[u][k], corners[v][k], offset));
                                }
                            }
                        }
                    }
                    index_cache.advance_cell();
                }
                index_cache.advance_row();
            }
            index_cache.advance_layer();

            self.layers.swap(0, 1);
        }
    }

}
