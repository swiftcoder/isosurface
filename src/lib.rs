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

//! Algorithms for extracting meshe data from isosurfaces.

/// Common math types
pub mod math;

/// Traits for defining isosurface data sources
pub mod source;

/// Convert isosurfaces to meshes using marching cubes.
///
/// Pros:
///
/// * Pretty fast.
/// * Extraction has no dependencies on neighbouring chunks.
///
/// Cons:
///
/// * Produces a lot of small triangle slivers
/// * Can't accurately reproduce sharp corners in the isosurface.
/// * Cracks between chunks of differing levels of detail
pub mod marching_cubes;

/// Convert isosurfaces to point clouds
///
/// Pros:
///
/// * Blindingly fast.
///
/// Cons:
///
/// * Doesn't contain any surface data. Surfaces have to be reconsutructed, probably on the GPU.
pub mod point_cloud;

/// Convert isosurfaces to meshes using marching cubes over a linear hashed octree.
///
/// This is a loose implementation of the paper [Fast Generation of Pointerless Octree Duals](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1467-8659.2010.01775.x).
///
/// Pros:
///
/// * Roughly twice as fast as standard marching cubes.
/// * Accurately reproduce sharp grid-aligned corners in the underlying isosurface.
///
/// Cons:
///
/// * Still can't accurately reproduce sharp corners which are not grid-aligned.
/// * Still no level-of-detail for neighbouring chunks.
pub mod linear_hashed_marching_cubes;

mod index_cache;
mod linear_hashed_octree;
mod marching_cubes_impl;
mod marching_cubes_tables;
mod morton;
