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
    feature::{PlaceFeatureInCell, TangentPlanes},
    math::Vec3,
};

/// The feature placement algorithm from [Efficient and Quality Contouring Algorithms on the GPU](https://doi.org/10.1111/j.1467-8659.2010.01825.x). This is a much simpler, and often faster alternative to the classic QEF minimisation traditionally used in Dual Contouring.
pub struct ParticleBasedMinimisation {}

const STEP_SIZE: f32 = 0.05;
const THRESHOLD: f32 = 0.02;

impl PlaceFeatureInCell for ParticleBasedMinimisation {
    fn place_feature_in_cell(&self, corners: &[Vec3; 8], normals: &[Vec3; 8]) -> Vec3 {
        let t = TangentPlanes::from_corners(corners, normals);
        let max_feature_size = corners[6].x - corners[0].x;

        let mut particle = t.center_of_mass;

        let mut forces = [Vec3::zero(); 8];
        for i in 0..8 {
            for j in 0..8 {
                forces[i] += corners[i] - t.planes[j].point_closest_to(corners[i]);
            }
        }

        for _ in 0..100 {
            let force = Self::trilinear(corners, &forces, particle) * STEP_SIZE * max_feature_size;

            particle += force;

            if force.len_sq() < THRESHOLD * max_feature_size {
                break;
            }
        }

        particle
    }
}

impl ParticleBasedMinimisation {
    fn trilinear(corners: &[Vec3; 8], forces: &[Vec3; 8], p: Vec3) -> Vec3 {
        // trilinear interpolation factor along each axis
        let f =
            (p - corners[0]) / (Vec3::new(corners[1].x, corners[3].y, corners[4].z) - corners[0]);

        // interpolate first along the x axis
        let c00 = (1.0 - f.x) * forces[0] + f.x * forces[1];
        let c01 = (1.0 - f.x) * forces[4] + f.x * forces[5];
        let c10 = (1.0 - f.x) * forces[3] + f.x * forces[2];
        let c11 = (1.0 - f.x) * forces[7] + f.x * forces[6];

        // Then along the y axis
        let c0 = (1.0 - f.y) * c00 + f.y * c10;
        let c1 = (1.0 - f.y) * c01 + f.y * c11;

        // Now finally along the z axis
        (1.0 - f.z) * c0 + f.z * c1
    }
}
