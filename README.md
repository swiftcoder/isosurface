# Isosurface
Isosurface extraction algorithms for Rust. Currently only a few techniques are implemented.

This crate intentionally has no dependencies to keep the footprint of the library small. The examples rely on the `glium`, `glium_text_rusttype`, and `cgmath` crates.

# Marching Cubes
The Marching Cubes implementation produces perfectly indexed meshes with few duplicate vertices, through the use of a (fairly involved) index caching system. The complexity of the cache could no doubt be reduced through some clever arithmetic, but it is not currently a bottleneck.

The implementation has been optimised for performance, with memory use kept as a low as possible considering. For an NxNxN voxel chunk, it will allocate roughly NxN of f32 storage for isosurface values, and Nx(N+1) of u32 storage for the index cache.

Indices are 32-bit because for chunks of 32x32 and larger you'll typically end up with more than 65k vertices. If you are targeting a mobile platform that supports only 16-bit indices, you'll need to use smaller chunk sizes, and truncate on the output side.

# Linear Hashed Marching Cubes
A very efficient algorithm using interleaved integer coordinates to represent octree cells, and storing them in a hash table. Results in better mesh quality than regular marching cubes, and is significantly faster. Memory usage is less predictable, but shouldn't be significantly higher than standard marching cubes.
 
# Point Clouds and Deferred Rasterisation
Point cloud extraction is typically not all that useful, given that point clouds don't contain any data about the actual surface. However, Gavan Woolery (gavanw@) posted an interesting image of reconstructing surface data in image space on the GPU, so I've added a simple example of that.  

# Why are optimisations enabled in debug builds?
Without optimisations enabled, debug builds are 70x slower (1 minute to extract a 256^3 volume, versus ~800 milliseconds). 

The marching cubes implementation relies on a lot of nested for loops over integer ranges, and the range iterators themselves entirely dominate the CPU profiles in unoptimised builds. While this could likely be worked around by converting the `for 0..8` style of loop to a while loop with manual counter, that seems ugly and distinctly not in the spirit of rust. I'd rather leave optimisations enabled, and wait for the compiler to become better at handling iterators.
