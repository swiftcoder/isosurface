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
mod particle_minimisation;
mod qef;

pub use particle_minimisation::*;
pub use qef::*;

use crate::{marching_cubes_tables::EDGE_CROSSING_MASK, math::Vec3};

const FEATURE_ANGLE: f32 = 0.8660254037844387; // cos(30º)

/// Place a mesh vertex at a feature point within a grid cell
pub trait PlaceFeatureInCell {
    /// Place a vertex as close as possible to any feature within the specified
    /// cell. Requires the corner vertices of the cell, and the normals at each
    /// corner.
    fn place_feature_in_cell(&self, corners: &[Vec3; 8], normals: &[Vec3; 8]) -> Vec3;
}

/// Classifies the topology of a surface within a specific region. Note that
/// since topology is evaluated based on discrete samples, it will not take into
/// account topological features that are smaller than the sampling frequency.
pub enum LocalTopology {
    /// The surface doesn't have any sharp features in this region.
    Planar,
    /// The surface has a sharp edge or crease in this region.
    Edge,
    /// The surface has a sharp corner in this region.
    Corner,
}

pub(crate) struct Plane {
    normal: Vec3,
    d: f32,
}

impl Plane {
    pub(crate) fn distance(&self, p: Vec3) -> f32 {
        self.normal.dot(p) - self.d
    }

    pub fn point_closest_to(&self, p: Vec3) -> Vec3 {
        let distance = self.distance(p);
        p - self.normal * distance
    }
}

/// The set of planes tangent to the surface within a given grid cell.
pub struct TangentPlanes {
    pub(crate) planes: Vec<Plane>,
    pub(crate) center_of_mass: Vec3,
    pub(crate) feature: LocalTopology,
}

impl TangentPlanes {
    /// Generate a set of tangent planes from the cell corners and the
    /// corresponding surface normals.
    pub fn from_corners(corners: &[Vec3; 8], normals: &[Vec3; 8]) -> Self {
        Self::new(corners, normals)
    }

    /// Generate a set of tangent planes from the points where the surface
    /// crosses the cell edges and the corresponding surface normals.
    pub fn from_edge_crossings(
        cube_index: usize,
        vertices: &[Vec3; 12],
        normals: &[Vec3; 12],
    ) -> Self {
        let edges = EDGE_CROSSING_MASK[cube_index];

        let indices: Vec<usize> = (0..12).filter(|i| (edges & (1 << i)) != 0).collect();

        let vertices: Vec<Vec3> = indices.iter().map(|&i| vertices[i]).collect();
        let normals: Vec<Vec3> = indices.iter().map(|&i| normals[i]).collect();

        Self::new(&vertices, &normals)
    }

    fn new(vertices: &[Vec3], normals: &[Vec3]) -> Self {
        let mut center_of_mass = Vec3::zero();
        let mut axis = Vec3::zero();
        let mut min_angle = std::f32::MAX;

        let mut count = 0.0;

        for i in 0..vertices.len() {
            for j in 0..vertices.len() {
                let angle = normals[i].dot(normals[j]);
                if angle < min_angle {
                    axis = normals[i].cross(normals[j]);
                    min_angle = angle;
                }
            }

            center_of_mass += vertices[i];
            count += 1.0;
        }

        center_of_mass /= count;

        let feature = if min_angle > FEATURE_ANGLE {
            LocalTopology::Planar
        } else {
            axis = axis.normalised().unwrap_or_default();
            let (mut min_c, mut max_c) = (1.0f32, -1.0f32);
            for &n in normals {
                let c = axis.dot(n);
                min_c = min_c.min(c);
                max_c = max_c.max(c);
            }
            let c = min_c.abs().max(max_c.abs());
            let c = (1.0 - c * c).sqrt();

            if c > FEATURE_ANGLE {
                LocalTopology::Edge
            } else {
                LocalTopology::Corner
            }
        };

        let planes = (0..vertices.len())
            .into_iter()
            .map(|i| {
                let normal = normals[i];
                let d = (vertices[i] - center_of_mass).dot(normal);

                Plane { normal, d }
            })
            .collect();

        Self {
            planes,
            center_of_mass,
            feature,
        }
    }
}
