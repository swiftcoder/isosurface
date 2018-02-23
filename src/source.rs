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

use math::Vec3;

/// A source capable of sampling a signed distance field at discrete coordinates.
pub trait Source {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return the signed distance (i.e. negative for coodinates inside the surface),
    /// as our Marching Cubes implementation will evaluate the surface at the zero-crossing.
    fn sample(&self, x: f32, y: f32, z: f32) -> f32;
}

/// A source capable of evaluating the normal vector to a signed distance field at discrete coordinates.
pub trait HermiteSource: Source {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return a normal vector to the surface.
    fn sample_normal(&self, x: f32, y: f32, z: f32) -> Vec3;
}

/// Adapts a `Source` to a `HermiteSource` by deriving normals from the surface via central differencing
pub struct CentralDifference<S>
where
    S: Source,
{
    source: S,
    epsilon: f32,
}

impl<S> CentralDifference<S>
where
    S: Source,
{
    /// Create an adaptor from a [Source](trait.Source.html)
    pub fn new(source: S) -> CentralDifference<S> {
        CentralDifference {
            source,
            epsilon: 0.0001,
        }
    }

    /// Create an adaptor from a [Source](trait.Source.html) and an epsilon value
    pub fn new_with_epsilon(source: S, epsilon: f32) -> CentralDifference<S> {
        CentralDifference { source, epsilon }
    }
}

impl<S> Source for CentralDifference<S>
where
    S: Source,
{
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        self.source.sample(x, y, z)
    }
}

impl<S> HermiteSource for CentralDifference<S>
where
    S: Source,
{
    fn sample_normal(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let v = self.sample(x, y, z);
        let vx = self.sample(x + self.epsilon, y, z);
        let vy = self.sample(x, y + self.epsilon, z);
        let vz = self.sample(x, y, z + self.epsilon);

        Vec3::new(vx - v, vy - v, vz - v)
    }
}
