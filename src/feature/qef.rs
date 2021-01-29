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
    feature::{LocalTopology, PlaceFeatureInCell, TangentPlanes},
    math::{svd::SVD, Vec3},
};

/// The feature placement algorithm used by Extended Marching Cubes and
/// traditional Dual Contouring. Uses Singular Value Decomposition to minimise
/// the quadratic error function defined by the tangent planes to the implicit
/// surface at the grid edge crossings.
pub struct MinimiseQEF {}

impl PlaceFeatureInCell for MinimiseQEF {
    fn place_feature_in_cell(&self, corners: &[Vec3; 8], normals: &[Vec3; 8]) -> Vec3 {
        let t = TangentPlanes::from_corners(corners, normals);
        Self::place_feature_with_tangents(&t)
    }
}

impl MinimiseQEF {
    /// Place a vertex as close as possible to any feature within the specified
    /// cell. Requires the tangent planes to the surface at the grid edge
    /// crossings.
    pub fn place_feature_with_tangents(t: &TangentPlanes) -> Vec3 {
        if let LocalTopology::Planar = t.feature {
            return t.center_of_mass;
        }

        let a: Vec<[f64; 3]> = t
            .planes
            .iter()
            .map(|p| [p.normal.x as f64, p.normal.y as f64, p.normal.z as f64])
            .collect();

        let mut svd = SVD::new(&a);

        // The system of equations is underspecified for edges, so
        // we zero the minimum singular value to reduce the rank
        if let LocalTopology::Edge = t.feature {
            let mut s_min = std::f64::MAX;
            let mut s_min_id = 0;

            for i in 0..3 {
                if svd.diagonal()[i] < s_min {
                    s_min = svd.diagonal()[i];
                    s_min_id = i;
                }
            }

            svd.diagonal()[s_min_id] = 0.0;
        }

        let b: Vec<f64> = t.planes.iter().map(|p| p.d as f64).collect();

        t.center_of_mass + svd.solve(&b)
    }
}
