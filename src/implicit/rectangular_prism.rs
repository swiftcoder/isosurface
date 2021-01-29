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
    math::Vec3,
    source::{HermiteSource, ScalarSource, VectorSource},
};
use std::f32::MAX;

/// A rectangular prism, or box.
#[derive(Copy, Clone)]
pub struct RectangularPrism {
    /// Half the extent of the box, the distance along each axis from the center
    /// point to the surface of the box.
    pub half_extent: Vec3,
}

impl RectangularPrism {
    /// Create a new rectangular prism from half the desired extents.
    pub fn new(half_extent: Vec3) -> Self {
        Self { half_extent }
    }
}

impl ScalarSource for RectangularPrism {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let q = p.abs() - self.half_extent;
        Signed(q.max(Vec3::zero()).len() + q.max_component().min(0.0))
    }
}

impl VectorSource for RectangularPrism {
    fn sample_vector(&self, p: Vec3) -> Directed {
        // Flip the point into the positive quadrant
        let a = p.abs();
        let mask = Vec3::new(
            if (a.yz() - self.half_extent.yz()).any(|f| f > 0.0) {
                1.0
            } else {
                -1.0
            },
            if (a.xz() - self.half_extent.xz()).any(|f| f > 0.0) {
                1.0
            } else {
                -1.0
            },
            if (a.xy() - self.half_extent.xy()).any(|f| f > 0.0) {
                1.0
            } else {
                -1.0
            },
        ) * MAX;
        // The closest point on a box is just the point clamped to the box bounds
        let closest_point_on_cube =
            if a.x < self.half_extent.x && a.y < self.half_extent.y && a.z < self.half_extent.z {
                a.max(self.half_extent)
            } else {
                a.min(self.half_extent)
            };
        // The distance along each axis is just the distance to the closest point
        Directed((a - closest_point_on_cube).max(mask))
    }
}

impl HermiteSource for RectangularPrism {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        p.clamp_to_cardinal_axis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangular_prism() {
        let prism = RectangularPrism::new(Vec3::new(1.0, 2.0, 4.0));

        assert_eq!(prism.sample_scalar(Vec3::zero()).0, -1.0);
        assert_eq!(prism.sample_scalar(Vec3::new(1.0, 2.0, 4.0)).0, 0.0);
        assert_eq!(prism.sample_scalar(Vec3::new(0.0, 0.0, 8.0)).0, 4.0);
        assert_eq!(prism.sample_scalar(Vec3::new(8.0, 0.0, 0.0)).0, 7.0);

        assert_eq!(
            prism.sample_vector(Vec3::zero()).0,
            Vec3::new(-1.0, -2.0, -4.0)
        );
        assert_eq!(
            prism.sample_vector(Vec3::new(1.0, 2.0, 4.0)).0,
            Vec3::zero()
        );
        assert_eq!(
            prism.sample_vector(Vec3::new(0.0, 0.0, 8.0)).0,
            Vec3::new(MAX, MAX, 4.0)
        );
        assert_eq!(
            prism.sample_vector(Vec3::new(8.0, 0.0, 0.0)).0,
            Vec3::new(7.0, MAX, MAX)
        );
        assert_eq!(
            prism.sample_vector(Vec3::new(8.0, 8.0, 8.0)).0,
            Vec3::from_scalar(MAX)
        );

        assert_eq!(
            prism
                .sample_normal(Vec3::new(0.0, 0.0, 8.0))
                .normalised()
                .unwrap(),
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            prism
                .sample_normal(Vec3::new(8.0, 0.0, 0.0))
                .normalised()
                .unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }
}
