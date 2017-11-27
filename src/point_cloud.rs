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
use marching_cubes_tables::CORNERS;

/// Extracts point clouds from distance fields.
pub struct PointCloud {
    size: usize,
    layers: [Vec<f32>; 2],
}

impl PointCloud {
    /// Create a new PointCloud with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size: usize) -> Self {
        PointCloud {
            size,
            layers: [vec![0f32; size * size], vec![0f32; size * size]],
        }
    }

    /// Extracts a point cloud from the given [`Source`](../source/trait.Source.html).
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the number of steps
    /// determined by the size provided to the constructor.
    ///
    /// The midpoints of extracted voxels will be appended to `vertices` as triples of (x, y, z)
    /// coordinates.
    pub fn extract_midpoints<S>(&mut self, source: &S, vertices: &mut Vec<f32>)
    where
        S: Source,
    {
        self.extract_impl(source, |x: f32, y: f32, z: f32| {
            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
        });
    }

    /// Extracts a point cloud with normal data from the given [`HermiteSource`](../source/trait.HermiteSource.html).
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the number of steps
    /// determined by the size provided to the constructor.
    ///
    /// The midpoints of extracted voxels will be appended to `vertices` as triples of (x, y, z)
    /// coordinates, followed by the surface normals as triples of (x, y, z) dimensions.
    pub fn extract_midpoints_with_normals<S>(&mut self, source: &S, vertices: &mut Vec<f32>)
    where
        S: HermiteSource,
    {
        self.extract_impl(source, |x: f32, y: f32, z: f32| {
            let n = source.sample_normal(x, y, z);
            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
            vertices.push(n.x);
            vertices.push(n.y);
            vertices.push(n.z);
        });
    }

    fn extract_impl<S, E>(&mut self, source: &S, mut extract: E)
    where
        S: Source,
        E: FnMut(f32, f32, f32) -> (),
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

        let mut values = [0f32; 8];

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

            // Extract the cells in the current layer
            for y in 0..size_minus_one {
                for x in 0..size_minus_one {
                    for i in 0..8 {
                        values[i] = self.layers[CORNERS[i][2]]
                            [(y + CORNERS[i][1]) * self.size + x + CORNERS[i][0]];
                    }

                    let mut cube_index = 0;
                    for i in 0usize..8 {
                        if values[i] <= 0.0 {
                            cube_index |= 1 << i;
                        }
                    }

                    if cube_index == 0 || cube_index == 255 {
                        continue;
                    }

                    let px = (x as f32 + 0.5) * one_over_size;
                    let py = (y as f32 + 0.5) * one_over_size;
                    let pz = (z as f32 + 0.5) * one_over_size;

                    extract(px, py, pz);
                }
            }

            self.layers.swap(0, 1);
        }
    }
}
