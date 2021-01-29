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

//! Algorithms for extracting mesh data from isosurfaces.

/// Common math types
pub mod math;

/// Traits for defining isosurface data sources
pub mod source;

/// Types for handling distances in different metric spaces.
pub mod distance;

/// Utilities for outputting mesh data in specific formats.
pub mod extractor;

/// Primitives for building distance fields from implicit functions.
pub mod implicit;

/// Sampling from distance fields.
pub mod sampler;

/// Algorithms for traversing bounded regions of distance fields.
pub mod traversal;

/// Algorithms for accurately placing vertices on features (edges or corners) of
/// an implicit surface.
pub mod feature;

mod dual_contouring;
mod extended_marching_cubes;
mod index_cache;
mod linear_hashed_marching_cubes;
mod linear_hashed_octree;
mod marching_cubes;
mod marching_cubes_impl;
mod marching_cubes_tables;
mod mesh;
mod morton;
mod point_cloud;

pub use self::{
    dual_contouring::*, extended_marching_cubes::*, linear_hashed_marching_cubes::*,
    marching_cubes::*, point_cloud::*,
};
