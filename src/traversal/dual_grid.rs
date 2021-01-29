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
    distance::Distance, marching_cubes_tables::CORNERS, math::Vec3, sampler::Sample,
    traversal::PrimalGrid,
};

/// Traverses over cubes in a dual grid. A dual grid is the grid formed by
/// placing a vertex in the center of each cube in a primal grid, and connecting
/// adjacent vertices into cubes. i.e. each 2x2x2 cube in the dual grid spans a
/// 3x3x3 region in the primal grid.
pub struct DualGrid<D: Distance> {
    size: usize,
    primal_grid: PrimalGrid<D>,
    duals: [Vec<(Vec3, D)>; 2],
}

impl<D: Distance> DualGrid<D> {
    /// Create a dual grid that spans a primal grid with dimensions NxNxN.
    /// The dual grid will have dimension (N-1) along each axis.
    pub fn new(size: usize) -> Self {
        let size_minus_one = size - 1;

        Self {
            size,
            primal_grid: PrimalGrid::new(size),
            duals: [
                vec![(Vec3::zero(), D::zero()); size_minus_one * size_minus_one],
                vec![(Vec3::zero(), D::zero()); size_minus_one * size_minus_one],
            ],
        }
    }

    /// Traverse the dual grid, sampling from the provided Sampler at each point
    /// in the primal grid. The vertex callback, if provided, will be
    /// invoked to adjust the location of each dual vertex, and provided
    /// with the vertices and distance samples corresponding to the enclosing
    /// primal cube. The cube callback will be invoked for each 2x2x2 set of
    /// neighbouring points in the dual grid, and provided the corner grid
    /// references, corner points, and the field values at those points.
    pub fn traverse<S, V, C>(
        &mut self,
        source: &S,
        mut vertex_callback: Option<V>,
        mut cube_callback: C,
    ) where
        S: Sample<D>,
        V: FnMut(&[Vec3; 8], &[D; 8]) -> Option<Vec3>,
        C: FnMut(&[(usize, usize, usize); 8], &[Vec3; 8], &[D; 8]),
    {
        let size_minus_one = self.size - 1;

        let mut keys = [(0, 0, 0); 8];
        let mut corners = [Vec3::zero(); 8];
        let mut values = [D::zero(); 8];

        // two separate borrows, so the borrow checker knows we aren't borrowing self
        // twice
        let primal_grid = &mut self.primal_grid;
        let duals = &mut self.duals;

        primal_grid.traverse(source, |primal_keys, primal_corners, primal_values| {
            let vertex = vertex_callback
                .as_mut()
                .and_then(|f| f(primal_corners, primal_values))
                .unwrap_or(primal_corners[0].lerp(primal_corners[6], 0.5));

            let (x, y, z) = primal_keys[0];
            duals[z % 2][y * size_minus_one + x] =
                (vertex, primal_values[0].lerp(primal_values[6], 0.5));

            if x > 0 && y > 0 && z > 0 {
                let (x, y, z) = (x - 1, y - 1, z - 1);
                for i in 0..8 {
                    keys[i] = (x + CORNERS[i][0], y + CORNERS[i][1], z + CORNERS[i][2]);
                    let dual = duals[(z + CORNERS[i][2]) % 2]
                        [(y + CORNERS[i][1]) * size_minus_one + x + CORNERS[i][0]];
                    corners[i] = dual.0;
                    values[i] = dual.1;
                }
                cube_callback(&keys, &corners, &values);
            }
        });
    }
}
