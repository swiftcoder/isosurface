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

//! Isosurface definitions for use in multiple examples

use isosurface::source::Source;

/// The distance-field equation for a torus
fn torus(x: f32, y: f32, z: f32) -> f32 {
    const R1: f32 = 1.0 / 4.0;
    const R2: f32 = 1.0 / 10.0;
    let q_x = ((x * x + y * y).sqrt()).abs() - R1;
    let len = (q_x * q_x + z * z).sqrt();
    len - R2
}

pub struct Torus {}

impl Source for Torus {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        torus(x - 0.5, y - 0.5, z - 0.5)
    }
}

fn abs(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    (
        if x > 0.0 { x } else { -x },
        if y > 0.0 { y } else { -y },
        if z > 0.0 { z } else { -z },
    )
}

fn max(px: f32, py: f32, pz: f32, qx: f32, qy: f32, qz: f32) -> (f32, f32, f32) {
    (
        if px > qx { px } else { qx },
        if py > qy { py } else { qy },
        if pz > qz { pz } else { qz },
    )
}

/// The distance field equation for a cube
fn cube(px: f32, py: f32, pz: f32, bx: f32, by: f32, bz: f32) -> f32 {
    let (ax, ay, az) = abs(px, py, pz);
    let (dx, dy, dz) = (ax - bx, ay - by, az - bz);
    let (mx, my, mz) = max(dx, dy, dz, 0.0, 0.0, 0.0);
    let l = (mx * mx + my * my + mz * mz).sqrt();
    dx.max(dz.max(dy)).min(0.0) + l
}

/// The distance field equation for a sphere
fn sphere(x: f32, y: f32, z: f32, r: f32) -> f32 {
    (x * x + y * y + z * z).sqrt() - r
}

/// Subtract one distance field from another (i.e. CSG difference operation)
fn subtract(d1: f32, d2: f32) -> f32 {
    d2.max(-d1)
}

pub struct CubeSphere {}

impl Source for CubeSphere {
    fn sample(&self, x: f32, y: f32, z: f32) -> f32 {
        subtract(
            sphere(x - 0.5, y - 0.5, z - 0.5, 0.25),
            cube(x - 0.5, y - 0.5, z - 0.5, 0.2, 0.2, 0.2),
        )
    }
}
