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
use crate::{math::Vec3, source::HermiteSource};

/// Trait for outputting mesh vertices and indices.
pub trait Extractor {
    fn extract_vertex(&mut self, vertex: Vec3);
    fn extract_index(&mut self, index: usize);
}

/// Output vertices as a tightly packed array of floats, discarding any face
/// data.
pub struct OnlyVertices<'a> {
    vertices: &'a mut Vec<f32>,
}

impl<'a> OnlyVertices<'a> {
    pub fn new(vertices: &'a mut Vec<f32>) -> Self {
        Self { vertices }
    }
}

impl<'a> Extractor for OnlyVertices<'a> {
    fn extract_vertex(&mut self, v: Vec3) {
        self.vertices.push(v.x);
        self.vertices.push(v.y);
        self.vertices.push(v.z);
    }

    fn extract_index(&mut self, _: usize) {}
}

/// Output vertices as a tightly packed array of floats, discarding any face
/// data.
pub struct OnlyInterleavedNormals<'a, S: HermiteSource> {
    vertices: &'a mut Vec<f32>,
    source: &'a S,
}

impl<'a, S: HermiteSource> OnlyInterleavedNormals<'a, S> {
    pub fn new(vertices: &'a mut Vec<f32>, source: &'a S) -> Self {
        Self { vertices, source }
    }
}

impl<'a, S: HermiteSource> Extractor for OnlyInterleavedNormals<'a, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.vertices.push(v.x);
        self.vertices.push(v.y);
        self.vertices.push(v.z);
        self.vertices.push(n.x);
        self.vertices.push(n.y);
        self.vertices.push(n.z);
    }

    fn extract_index(&mut self, _: usize) {}
}

/// Output vertices as a tightly packed array of floats.
pub struct IndexedVertices<'a> {
    vertices: &'a mut Vec<f32>,
    indices: &'a mut Vec<u32>,
}

impl<'a> IndexedVertices<'a> {
    pub fn new(vertices: &'a mut Vec<f32>, indices: &'a mut Vec<u32>) -> Self {
        Self { vertices, indices }
    }
}

impl<'a> Extractor for IndexedVertices<'a> {
    fn extract_vertex(&mut self, v: Vec3) {
        self.vertices.push(v.x);
        self.vertices.push(v.y);
        self.vertices.push(v.z);
    }

    fn extract_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
}

/// Sample normals from an implicit surface and output them interleaved with
/// vertices, as a tightly packed array of floats.
pub struct IndexedInterleavedNormals<'a, S: HermiteSource> {
    vertices: &'a mut Vec<f32>,
    indices: &'a mut Vec<u32>,
    source: &'a S,
}

impl<'a, S: HermiteSource> IndexedInterleavedNormals<'a, S> {
    pub fn new(vertices: &'a mut Vec<f32>, indices: &'a mut Vec<u32>, source: &'a S) -> Self {
        Self {
            vertices,
            indices,
            source,
        }
    }
}

impl<'a, S: HermiteSource> Extractor for IndexedInterleavedNormals<'a, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.vertices.push(v.x);
        self.vertices.push(v.y);
        self.vertices.push(v.z);
        self.vertices.push(n.x);
        self.vertices.push(n.y);
        self.vertices.push(n.z);
    }

    fn extract_index(&mut self, index: usize) {
        self.indices.push(index as u32);
    }
}
