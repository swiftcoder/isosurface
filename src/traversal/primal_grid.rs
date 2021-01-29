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
use crate::{distance::Distance, marching_cubes_tables::CORNERS, math::Vec3, sampler::Sample};

/// Traverses over cubes in a primal grid (i.e. cubes formed by adjacent sample
/// points).
pub struct PrimalGrid<D: Distance> {
    size: usize,
    layers: [Vec<(Vec3, D)>; 2],
}

impl<D: Distance> PrimalGrid<D> {
    /// Create a cubic grid with dimensions N*N*N
    pub fn new(size: usize) -> Self {
        Self {
            size,
            layers: [
                vec![(Vec3::zero(), D::zero()); size * size],
                vec![(Vec3::zero(), D::zero()); size * size],
            ],
        }
    }

    /// Traverse the primal grid, sampling from the provided Sampler at each
    /// grid point. The callback will be invoked for each 2x2x2 set of
    /// neighbouring grid points, and provided the corner grid references,
    /// corner points, and the field values at those points.
    pub fn traverse<S, C>(&mut self, source: &S, mut callback: C)
    where
        S: Sample<D>,
        C: FnMut(&[(usize, usize, usize); 8], &[Vec3; 8], &[D; 8]),
    {
        let size_minus_one = self.size - 1;
        let one_over_size = 1.0 / (size_minus_one as f32);

        // Cache layer zero of distance field values
        for y in 0usize..self.size {
            for x in 0..self.size {
                let corner = Vec3::new(x as f32 * one_over_size, y as f32 * one_over_size, 0.0);
                self.layers[0][y * self.size + x] = (corner, source.sample(corner));
            }
        }

        let mut keys = [(0, 0, 0); 8];
        let mut corners = [Vec3::zero(); 8];
        let mut values = [D::zero(); 8];

        for z in 0..self.size {
            // Cache layer N+1 of isosurface values
            for y in 0..self.size {
                for x in 0..self.size {
                    let corner = Vec3::new(
                        x as f32 * one_over_size,
                        y as f32 * one_over_size,
                        (z + 1) as f32 * one_over_size,
                    );
                    self.layers[1][y * self.size + x] = (corner, source.sample(corner));
                }
            }

            // Traverse the calls in the current layer
            for y in 0..size_minus_one {
                for x in 0..size_minus_one {
                    for i in 0..8 {
                        keys[i] = (x + CORNERS[i][0], y + CORNERS[i][1], z + CORNERS[i][2]);
                        let (corner, value) = self.layers[CORNERS[i][2]]
                            [(y + CORNERS[i][1]) * self.size + x + CORNERS[i][0]];
                        corners[i] = corner;
                        values[i] = value;
                    }

                    callback(&keys, &corners, &values);
                }
            }

            self.layers.swap(0, 1);
        }
    }
}
