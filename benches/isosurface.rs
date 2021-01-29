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

use criterion::{criterion_group, criterion_main, Criterion};
use isosurface::{
    distance::Signed, extractor::IndexedVertices, implicit::Torus, sampler::Sampler,
    LinearHashedMarchingCubes, MarchingCubes,
};

fn marching_cubes() {
    let torus = Torus::new(0.25, 0.1);
    let sampler = Sampler::new(&torus);

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut extractor = IndexedVertices::new(&mut vertices, &mut indices);

    let mut marching_cubes = MarchingCubes::<Signed>::new(128);
    marching_cubes.extract(&sampler, &mut extractor);
}

fn linear_hashed_marching_cubes() {
    let torus = Torus::new(0.25, 0.1);
    let sampler = Sampler::new(&torus);

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut extractor = IndexedVertices::new(&mut vertices, &mut indices);

    let mut marching_cubes = LinearHashedMarchingCubes::new(7);
    marching_cubes.extract(&sampler, &mut extractor);
}

fn marching_cubes_benchmark(c: &mut Criterion) {
    c.bench_function("marching cubes", |b| b.iter(|| marching_cubes()));
    c.bench_function("linear hashed marching cubes", |b| {
        b.iter(|| linear_hashed_marching_cubes())
    });
}

criterion_group!(benches, marching_cubes_benchmark);
criterion_main!(benches);
