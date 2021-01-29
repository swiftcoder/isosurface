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
    distance::Distance, extractor::Extractor, marching_cubes_impl::classify_corners,
    sampler::Sample, traversal::PrimalGrid,
};

/// Convert isosurfaces to point clouds
///
/// Pros:
///
/// * Blindingly fast.
///
/// Cons:
///
/// * Doesn't contain any surface data. Surfaces have to be reconstructed,
///   ideally on the GPU itself.
pub struct PointCloud<D: Distance> {
    primal_grid: PrimalGrid<D>,
}

impl<D: Distance> PointCloud<D> {
    /// Create a new PointCloud with the given chunk size.
    ///
    /// For a given `size`, this will evaluate chunks of `size^3` voxels.
    pub fn new(size: usize) -> Self {
        PointCloud {
            primal_grid: PrimalGrid::new(size),
        }
    }

    /// Extracts a point cloud from the given [Sample].
    ///
    /// The Source will be sampled in the range (0,0,0) to (1,1,1), with the
    /// number of steps determined by the size provided to the constructor.
    ///
    /// The resulting vertex data will be returned via the provided
    /// Extractor. Note that no face data will be produced.
    pub fn extract<S, E>(&mut self, source: &S, extractor: &mut E)
    where
        S: Sample<D>,
        E: Extractor,
    {
        self.primal_grid.traverse(source, |_keys, corners, values| {
            let cube_index = classify_corners(&values);

            if cube_index != 0 && cube_index != 255 {
                let p = corners[0].lerp(corners[6], 0.5);
                extractor.extract_vertex(p);
            }
        });
    }
}
