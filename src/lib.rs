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

//! Algorithms for extracting meshes from isosurfaces.

/// Traits for defining isosurface data sources
pub mod source;

/// Convert isosurfaces to meshes using marching cubes.
///
/// Pros:
///
/// * Very fast
/// * Extraction has no dependencies on neighbouring chunks
///
/// Cons:
///
/// * Produces a lot of small triangle slivers
/// * Cracks between chunks of differing levels of detail
pub mod marching_cubes;

/// Convert isosurfaces to point clouds
///
/// Pros:
///
/// * Blindingly fast
///
/// Cons:
///
/// * Doesn't contain any surface data. Surfaces have to be reconsutructed, probably on the GPU.
pub mod point_cloud;

mod marching_cubes_tables;
mod index_cache;
