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
    distance::{Directed, Signed},
    math::{Vec2, Vec3},
    source::{HermiteSource, ScalarSource, VectorSource},
};
use std::f32::MAX;

/// A torus, or doughnut-shape.
#[derive(Copy, Clone)]
pub struct Torus {
    /// The radius from the center point to the middle of the outer ring.
    pub radius: f32,
    /// The radius of the outer ring itself.
    pub tube_radius: f32,
}

impl Torus {
    /// Create a new torus from the primary radius and radius of the outer ring.
    pub fn new(radius: f32, tube_radius: f32) -> Self {
        Self {
            radius,
            tube_radius,
        }
    }
}

impl ScalarSource for Torus {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let q_x = ((p.x * p.x + p.y * p.y).sqrt()).abs() - self.radius;
        let len = (q_x * q_x + p.z * p.z).sqrt();
        Signed(len - self.tube_radius)
    }
}

impl VectorSource for Torus {
    fn sample_vector(&self, p: Vec3) -> Directed {
        // Flip the point into the positive quadrant
        let a = p.abs();
        // // Find the closest point on the major radius of the torus
        // let point_in_ring = Vec3::new(a.x, a.y, 0.0)
        //     .normalised()
        //     .unwrap_or_else(|| Vec3::new(1.0, 1.0, 0.0))
        //     * self.radius;

        // // Transform the input point relative to the new point, and flip it into the
        // positive quadrant again let a = (a - point_in_ring).abs();
        // // Find the closest point on the minor radius of the torus
        // let closest_point_on_ring =
        //     a.normalised().unwrap_or(Vec3::new(0.0, 0.0, 1.0)) * self.tube_radius;
        // // The distance along each axis is just the distance to the closest point
        // Directed(a - closest_point_on_ring);

        let xy = Vec2::new(a.x, a.y);
        let l = xy.len();
        let l_xy = l - self.radius;
        let tube_radius_at_z = (self.tube_radius * self.tube_radius - a.z * a.z).sqrt();
        let r = Vec2::new(
            self.radius + tube_radius_at_z,
            self.radius - tube_radius_at_z,
        );
        Directed(Vec3::new(
            if a.z > self.tube_radius || a.y > self.radius + tube_radius_at_z {
                MAX
            } else if a.x == 0.0 {
                (a.y - self.radius).abs() - self.tube_radius
            } else {
                (a.x - (r.x * r.x - a.y * a.y).sqrt()).max((r.y * r.y - a.y * a.y).sqrt() - a.x)
            },
            if a.z > self.tube_radius || a.x > self.radius + tube_radius_at_z {
                MAX
            } else if a.y == 0.0 {
                (a.x - self.radius).abs() - self.tube_radius
            } else {
                (a.y - (r.x * r.x - a.x * a.x).sqrt()).max((r.y * r.y - a.x * a.x).sqrt() - a.y)
            },
            if l_xy.abs() > self.tube_radius {
                MAX
            } else {
                a.z - (self.tube_radius * self.tube_radius - l_xy * l_xy).sqrt()
            },
        ))
    }
}

impl HermiteSource for Torus {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        // Find the closest point on the major radius of the torus
        let point_in_ring = Vec3::new(p.x, p.y, 0.0)
            .normalised()
            .unwrap_or(Vec3::new(1.0, 0.0, 0.0))
            * self.radius;

        p - point_in_ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torus() {
        let torus = Torus::new(8.0, 2.0);

        assert_eq!(torus.sample_scalar(Vec3::zero()).0, 6.0);
        assert_eq!(torus.sample_scalar(Vec3::new(8.0, 0.0, 0.0)).0, -2.0);
        assert_eq!(torus.sample_scalar(Vec3::new(10.0, 0.0, 0.0)).0, 0.0);
        assert_eq!(torus.sample_scalar(Vec3::new(12.0, 0.0, 0.0)).0, 2.0);
        assert_eq!(torus.sample_scalar(Vec3::new(8.0, 0.0, 8.0)).0, 6.0);

        assert_eq!(
            torus.sample_vector(Vec3::zero()).0,
            Vec3::new(6.0, 6.0, MAX)
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(8.0, 0.0, 0.0)).0,
            Vec3::from_scalar(-2.0),
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(10.0, 0.0, 0.0)).0,
            Vec3::zero()
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(12.0, 0.0, 0.0)).0,
            Vec3::new(2.0, MAX, MAX)
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(12.0, 12.0, 0.0)).0,
            Vec3::from_scalar(MAX)
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(9.0, 9.0, 0.0)).0,
            Vec2::from_scalar(9.0 - (10.0 * 10.0 - 9.0 * 9.0f32).sqrt()).extend(MAX)
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(2.0, 2.0, 0.0)).0,
            Vec2::from_scalar((6.0 * 6.0 - 2.0 * 2.0f32).sqrt() - 2.0).extend(MAX)
        );
        assert_eq!(
            torus.sample_vector(Vec3::new(8.0, 0.0, 8.0)).0,
            Vec3::new(MAX, MAX, 6.0)
        );

        assert_eq!(
            torus
                .sample_normal(Vec3::new(8.0, 0.0, 8.0))
                .normalised()
                .unwrap(),
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            torus
                .sample_normal(Vec3::new(12.0, 0.0, 0.0))
                .normalised()
                .unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }
}
