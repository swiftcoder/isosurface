# Isosurface
Isosurface extraction algorithms for Rust. Currently only Marching Cubes is implemented, fancier algorithms to be added at a later date.

This crate has no dependencies, although the example relies on the glium, cgmath, num crates.

# Marching Cubes
The Marching Cubes implementation produces perfectly indexed meshes with no duplicate vertices, through the use of a (fairly involved) index caching system. The complexity of the cache could no doubt be reduced through some clever arithmetic, but it is not currently a bottleneck.

The implementation has been optimised for performance, with memory use kept as a low as possible considering. For an NxNxN voxel chunk, it will allocate roughly NxN of f32 storage for isosurface values, and Nx(N+1) of u32 storage for the index cache.

Indices are 32-bit becuase for chunks of 32x32 and larger you'll typically end up with mor than 65k vertices. If you are targetting a mobile platform that supports only 16-bit indices, you'll need to use smaller chunk sizes, and truncate on the output side.

# Why are optimisations enabled in debug builds?
Without optimisations enabled, debug builds are 70x slower (1 minute to extract a 256^3 volume, versus ~800 milliseconds). 

This implementation relies on a lot of nested for loops over integer ranges, and the range iterators themselves entirely dominate the CPU profiles in unoptimised builds. While this could likely be worked around by converting the `for 0..8` style of loop to a while loop with manual counter, that seems ugly and distinctly not in the spirit of rust. I'd rather leave optimisations enabled, and wait for the compiler to become better at handling iterators.
