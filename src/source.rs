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
