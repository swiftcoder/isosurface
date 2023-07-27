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
};

/// A source capable of sampling a signed distance field at discrete
/// coordinates.
pub trait ScalarSource {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return the signed distance (i.e. negative for coordinates inside
    /// the surface), as our Marching Cubes implementation will evaluate the
    /// surface at the zero-crossing.
    fn sample_scalar(&self, p: Vec3) -> Signed;
}

/// A source capable of sampling a directed distance field at discrete
/// coordinates.
pub trait VectorSource {
    /// Samples the directed distance field at the given (x, y, z) coordinates.
    ///
    /// Must return the signed distance (i.e. negative for coordinates inside
    /// the surface), as our Marching Cubes implementation will evaluate the
    /// surface at the zero-crossing.
    fn sample_vector(&self, p: Vec3) -> Directed;
}

/// A source capable of evaluating the normal vector to a distance field
/// at discrete coordinates.
pub trait HermiteSource: ScalarSource {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return a normal vector to the surface.
    fn sample_normal(&self, p: Vec3) -> Vec3;
}

/// Adapts a [ScalarSource] to a [HermiteSource] by deriving normals from the
/// surface via central differencing.
pub struct CentralDifference<S: ScalarSource> {
    pub source: S,
    epsilon: f32,
}

impl<S: ScalarSource> CentralDifference<S> {
    /// Create an adaptor from a [ScalarSource].
    pub fn new(source: S) -> Self {
        Self::new_with_epsilon(source, 0.000001)
    }

    /// Create an adaptor from a [ScalarSource] and an epsilon
    /// value.
    pub fn new_with_epsilon(source: S, epsilon: f32) -> Self {
        Self { source, epsilon }
    }
}

impl<S: ScalarSource> ScalarSource for CentralDifference<S> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        self.source.sample_scalar(p)
    }
}

impl<S: VectorSource + ScalarSource> VectorSource for CentralDifference<S> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        self.source.sample_vector(p)
    }
}

impl<S: ScalarSource> HermiteSource for CentralDifference<S> {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        let dx = Vec3::new(self.epsilon, 0.0, 0.0);
        let vx = self.sample_scalar(p + dx).0 - self.sample_scalar(p - dx).0;

        let dy = Vec3::new(0.0, self.epsilon, 0.0);
        let vy = self.sample_scalar(p + dy).0 - self.sample_scalar(p - dy).0;

        let dz = Vec3::new(0.0, 0.0, self.epsilon);
        let vz = self.sample_scalar(p + dz).0 - self.sample_scalar(p - dz).0;

        Vec3::new(vx, vy, vz) / (2.0 * self.epsilon)
    }
}
