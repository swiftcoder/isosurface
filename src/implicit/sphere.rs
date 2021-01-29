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

/// A sphere.
#[derive(Copy, Clone)]
pub struct Sphere {
    /// The radius of the sphere.
    pub radius: f32,
}

impl Sphere {
    /// Create a new sphere from the desired radius.
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl ScalarSource for Sphere {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(p.len() - self.radius)
    }
}

impl VectorSource for Sphere {
    fn sample_vector(&self, p: Vec3) -> Directed {
        // Flip the point into the positive quadrant
        let a = p.abs();

        let r2 = self.radius * self.radius;
        let l_yz = r2 - a.yz().len_sq();
        let l_xz = r2 - a.xz().len_sq();
        let l_xy = r2 - a.xy().len_sq();

        Directed(Vec3::new(
            if l_yz < 0.0 { MAX } else { a.x - l_yz.sqrt() },
            if l_xz < 0.0 { MAX } else { a.y - l_xz.sqrt() },
            if l_xy < 0.0 { MAX } else { a.z - l_xy.sqrt() },
        ))
    }
}

impl HermiteSource for Sphere {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere() {
        let sphere = Sphere::new(2.0);

        assert_eq!(sphere.sample_scalar(Vec3::zero()).0, -2.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(2.0, 0.0, 0.0)).0, 0.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(0.0, 0.0, 8.0)).0, 6.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(8.0, 0.0, 0.0)).0, 6.0);

        assert_eq!(
            sphere.sample_vector(Vec3::zero()).0,
            Vec3::from_scalar(-2.0)
        );
        assert_eq!(
            sphere.sample_vector(Vec3::new(2.0, 0.0, 0.0)).0,
            Vec3::zero()
        );
        assert_eq!(
            sphere.sample_vector(Vec3::new(0.0, 0.0, 8.0)).0,
            Vec3::new(MAX, MAX, 6.0)
        );
        assert_eq!(
            sphere.sample_vector(Vec3::new(8.0, 0.0, 0.0)).0,
            Vec3::new(6.0, MAX, MAX)
        );

        assert_eq!(
            sphere
                .sample_normal(Vec3::new(0.0, 0.0, 8.0))
                .normalised()
                .unwrap(),
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            sphere
                .sample_normal(Vec3::new(8.0, 0.0, 0.0))
                .normalised()
                .unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }
}
