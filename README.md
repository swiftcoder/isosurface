# Isosurface
Isosurface extraction algorithms implemented in Rust. The classical Marching Cubes and Dual Contouring techniques are included, along with more modern variations on the theme.

In the interest of education, the documentation for each extraction algorithm links to the relevant academic papers.

## Example programs
`cargo run --example sampler` will execute the sampler, which allows you to compare a variety of algorithms and implicit surfaces.

 `cargo run --example deferred_rasterisation` will execute a demonstration of GPU-side deferred rasterisation from point clouds. This is a technique pioneered by Gavan Woolery, of [Voxel Quest](https://www.voxelquest.com) fame.

## Dependencies
This library intentionally has no dependencies. While that requires some redevelopment of common code (i.e. the Vec3 type), it keeps the footprint of the library small, and compile times low for consuming crates. The examples do however rely on the `glium`, `glium_text_rusttype`, and `cgmath` crates, to avoid reinventing the world.

## 32-bit indices
For simplicity vertex indices have been fixed at 32-bits, because for chunks of 32x32x32 and larger you'll often end up with more than 65k vertices. If you are targeting a mobile platform that supports only 16-bit indices, you'll need to keep to smaller chunk sizes, or split the mesh on the output side.

## Why are optimisations enabled in debug builds?
Without optimisations enabled, debug builds are around 70x slower. The implementation relies on a lot of nested for loops over integer ranges, and the range iterators themselves entirely dominate the CPU profiles in unoptimised builds.

While this can be worked around by converting the `for 0..8` style of loop to a while loop with manual counter, the result is quite unpleasant, and distinctly not in the spirit of rust. I'd rather leave optimisations enabled, and wait for the compiler to become better at handling iterators in debug builds.

If you take a dependency on this crate and run into the same issue, you can tell Cargo to compile just this one crate in release mode, by adding the following to your `Cargo.toml`:

```
[profile.dev.package.isosurface]
opt-level = 3
```