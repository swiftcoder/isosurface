use criterion::{criterion_group, criterion_main, Criterion};
use isosurface::{
    linear_hashed_marching_cubes::LinearHashedMarchingCubes, marching_cubes::MarchingCubes,
    source::Source,
};

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

fn marching_cubes() {
    let torus = Torus {};
    let mut vertices = vec![];
    let mut indices = vec![];

    let mut marching_cubes = MarchingCubes::new(256);
    marching_cubes.extract(&torus, &mut vertices, &mut indices);
}

fn linear_hashed_marching_cubes() {
    let torus = Torus {};
    let mut vertices = vec![];
    let mut indices = vec![];

    let mut marching_cubes = LinearHashedMarchingCubes::new(8);
    marching_cubes.extract(&torus, &mut vertices, &mut indices);
}

fn marching_cubes_benchmark(c: &mut Criterion) {
    c.bench_function("marching cubes", |b| b.iter(|| marching_cubes()));
    c.bench_function("linear hashed marching cubes", |b| {
        b.iter(|| linear_hashed_marching_cubes())
    });
}

criterion_group!(benches, marching_cubes_benchmark);
criterion_main!(benches);
