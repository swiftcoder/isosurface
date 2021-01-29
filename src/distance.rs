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
use crate::math::Vec3;

// Used to compute the diagonal dimension (i.e. 3-dimensional hypotenuse) of a
// cube.
const SQRT_OF_3: f32 = 1.732_050_807_57;

/// A representation of distance in a specific metric space.
pub trait Distance: Copy + Clone {
    /// Create a zero distance.
    fn zero() -> Self;

    /// Determine if the given distance is positive.
    fn is_positive(&self) -> bool;

    /// Interpolate between two distances.
    fn lerp(&self, other: Self, factor: f32) -> Self;

    /// Test if the distance is within a cube of the specified amount.
    fn within_extent(&self, extent: f32) -> bool;

    /// Find the point along the line between the given grid points,
    /// that lies at the zero-crossing of the associated distances.
    fn find_crossing_point(a: Self, b: Self, p_a: Vec3, p_b: Vec3) -> Vec3;
}

/// A signed scalar distance.
#[derive(Copy, Clone)]
pub struct Signed(pub f32);

/// A signed distance along all three cardinal axis.
#[derive(Copy, Clone)]
pub struct Directed(pub Vec3);

impl Distance for Signed {
    fn zero() -> Self {
        Signed(0.0)
    }

    fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    fn lerp(&self, other: Self, f: f32) -> Self {
        Signed((1.0 - f) * self.0 + f * other.0)
    }

    fn within_extent(&self, extent: f32) -> bool {
        self.0.abs() < extent * SQRT_OF_3
    }

    fn find_crossing_point(a: Self, b: Self, p_a: Vec3, p_b: Vec3) -> Vec3 {
        let delta = b.0 - a.0;
        let t = if delta == 0.0 { 0.5 } else { -a.0 / delta };

        p_a * (1.0 - t) + p_b * t
    }
}

impl Distance for Directed {
    fn zero() -> Self {
        Directed(Vec3::zero())
    }

    fn is_positive(&self) -> bool {
        // If any component is positive, we're not inside the surface
        self.0.x > 0.0 || self.0.y > 0.0 || self.0.z > 0.0
    }

    fn lerp(&self, other: Self, f: f32) -> Self {
        Directed(self.0.lerp(other.0, f))
    }

    fn within_extent(&self, extent: f32) -> bool {
        self.0.abs().any(|f| f < extent * SQRT_OF_3)
    }

    fn find_crossing_point(a: Self, b: Self, p_a: Vec3, p_b: Vec3) -> Vec3 {
        // Since we're working on a grid, we only care about distance along the dominant
        // axis
        let axis = (p_a - p_b).abs().max_component_index();

        let delta = b.0[axis] - a.0[axis];
        let t = if delta == 0.0 {
            0.5
        } else {
            -a.0[axis] / delta
        };

        p_a * (1.0 - t) + p_b * t
    }
}
