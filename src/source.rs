// Copyright 2017 Tristam MacDonald
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

/// A source capable of sampling a signed distance field at discrete coordinates.
pub trait Source {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return the signed distance (i.e. negative for coodinates inside the surface),
    /// as our Marching Cubes implementation will evaluate the surface at the zero-crossing.
    fn sample(&self, x : f32, y : f32, z : f32) -> f32;
}

/// A source capable of evaluating the normal vector to a signed distance field at discrete coordinates.
pub trait HermiteSource : Source {
    /// Samples the distance field at the given (x, y, z) coordinates.
    ///
    /// Must return a normal vector to the surface as an (x, y, z) tuple.
    fn sample_normal(&self, x : f32, y : f32, z : f32) -> (f32, f32, f32);
}

/// Adapts a Source to a HermiteSource by deriving normals from the surface via central differencing
pub struct CentralDifference {
    source : Box<Source>,
    epsilon : f32,
}

impl CentralDifference {
    /// Create an adaptor from a [Source](trait.Source.html)
    pub fn new(source : Box<Source>) -> CentralDifference {
        CentralDifference{
            source,
            epsilon: 0.0001,
        }
    }

    /// Create an adaptor from a [Source](trait.Source.html) and an epsilon value
    pub fn new_with_epsilon(source : Box<Source>, epsilon : f32) -> CentralDifference {
        CentralDifference {
            source,
            epsilon,
        }
    }
}

impl Source for CentralDifference {
    fn sample(&self, x : f32, y : f32, z : f32) -> f32 {
        self.source.sample(x, y, z)
    }
}

impl HermiteSource for CentralDifference {
    fn sample_normal(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let v = self.sample(x, y, z);
        let vx = self.sample(x + self.epsilon, y, z);
        let vy = self.sample(x, y + self.epsilon, z);
        let vz = self.sample(x, y, z + self.epsilon);

        (vx - v, vy - v, vz - v)
    }
}
