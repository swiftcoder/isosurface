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

//! Isosurface definitions for use in multiple examples
use isosurface::{
    distance::{Directed, Signed},
    math::Vec3,
    source::{HermiteSource, ScalarSource, VectorSource},
};

pub trait AllSources: ScalarSource + VectorSource + HermiteSource {}

impl<S: ScalarSource + VectorSource + HermiteSource> AllSources for S {}

pub struct DemoSource<'a> {
    pub source: Box<dyn 'a + AllSources>,
}

impl<'a> DemoSource<'a> {
    pub fn new<S: 'a + AllSources>(source: S) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

impl<'a> ScalarSource for DemoSource<'a> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let q = p - Vec3::from_scalar(0.5);
        self.source.sample_scalar(q)
    }
}

impl<'a> VectorSource for DemoSource<'a> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        let q = p - Vec3::from_scalar(0.5);
        self.source.sample_vector(q)
    }
}

impl<'a> HermiteSource for DemoSource<'a> {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        let q = p - Vec3::from_scalar(0.5);
        self.source.sample_normal(q)
    }
}
