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
use crate::{extractor::Extractor, index_cache::IndexCache, math::Vec3};
use std::{
    collections::{hash_set::Iter as HashSetIter, HashMap, HashSet},
    hash::Hash,
};

/// A handle to a specific vertex within a vertex array
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VertexHandle(usize);

/// A handle to a specific face within a mesh
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FaceHandle(usize);

/// A face within a mesh. Faces are required to be triangles (i.e. composed of
/// exactly 3 vertices and 3 edges)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Face([VertexHandle; 3]);

/// An edge within a mesh. Edges are bidirectional (i.e. Edge(u,v) and Edge(v,u)
/// represent the same edge)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Edge(VertexHandle, VertexHandle);

/// Utility class to store and manipulate the topology of meshes
///
/// Note that this type does not store the mesh vertices themselves.
/// It is expected that the caller will maintain the actual vertex
/// data, and use this class to generate a suitable set of indices
/// for the mesh topology.
pub struct MeshTopology {
    next_vertex: usize,
    faces: Vec<Face>,
    edges: HashSet<Edge>,
    edge_to_face: HashMap<Edge, Vec<FaceHandle>>,
}

impl MeshTopology {
    /// Create a new empty MeshTopology
    pub fn new() -> Self {
        Self {
            next_vertex: 0,
            faces: vec![],
            edges: HashSet::new(),
            edge_to_face: HashMap::new(),
        }
    }

    /// Allocate a new vertex handle. The caller is responsible for
    /// storing the actual vertex data associated with this handle.
    pub fn add_vertex(&mut self) -> VertexHandle {
        let handle = VertexHandle(self.next_vertex);
        self.next_vertex += 1;
        handle
    }

    /// Add a new face, given 3 vertices in counter-clockwise order.
    pub fn add_face(&mut self, a: VertexHandle, b: VertexHandle, c: VertexHandle) -> FaceHandle {
        let face = FaceHandle(self.faces.len());
        self.faces.push(Face([a, b, c]));

        let edge_a = Edge::new(a, b);
        self.edges.insert(edge_a);
        self.edge_to_face.entry(edge_a).or_default().push(face);

        let edge_b = Edge::new(b, c);
        self.edges.insert(edge_b);
        self.edge_to_face.entry(edge_b).or_default().push(face);

        let edge_c = Edge::new(c, a);
        self.edges.insert(edge_c);
        self.edge_to_face.entry(edge_c).or_default().push(face);

        face
    }

    /// Build an index buffer from the mesh, suitable for use by rendering APIs
    pub fn extract_indices<E>(&self, extractor: &mut E)
    where
        E: Extractor,
    {
        for face in &self.faces {
            for v in &face.0 {
                extractor.extract_index(v.0);
            }
        }
    }

    /// An iterator over the unique edges in the mesh
    pub fn edges(&self) -> HashSetIter<Edge> {
        self.edges.iter()
    }

    /// The faces that share a given edge. In an ideal world, meshes would be
    /// manifold, and at most 2 faces would share a single edge. However
    /// isosurface extraction may produce non-manifold meshes with 3 or more
    /// faces sharing an edge.
    pub fn adjoining_faces(&self, edge: Edge) -> Vec<Face> {
        self.edge_to_face
            .get(&edge)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|f| self.faces[f.0])
            .collect()
    }

    /// Rotate an edge within the mesh.
    ///
    /// Given a pair of faces which share the specified edge, this will
    /// flip the direction of the common edge, like so:
    /// ```text
    /// *---*      *---*
    /// |\  |      |  /|
    /// | \ |  ==> | / |
    /// |  \|      |/  |
    /// *---*      *---*
    /// ```
    /// This is useful when the two faces are not co-planar (i.e. the edge
    /// represents a crease), and the edge currently runs in the opposite
    /// direction to the crease.
    ///
    /// Edge rotation only works for edges that are shared by exactly 2 faces,
    /// so we silently ignore requests to rotate other types of edge.
    pub fn rotate_edge(&mut self, edge: Edge) {
        if let Some(adjoining) = self.edge_to_face.get(&edge) {
            // Only rotate if the edge is adjoining exactly 2 faces
            if let [handle_a, handle_b] = adjoining[..] {
                let face_a = self.faces[handle_a.0];
                let face_b = self.faces[handle_b.0];

                // Find the two vertices that aren't on the shared edge
                let c = face_a.vertex_opposite(edge);
                let d = face_b.vertex_opposite(edge);

                // We don't know which way the edge runs, so use the vertex winding
                // to determine if we need to flip it
                let (u, v) = if face_a.matches_winding_direction(edge) {
                    (edge.end(), edge.start())
                } else {
                    (edge.start(), edge.end())
                };

                // Overwrite the two faces with the two new faces
                self.faces[handle_a.0].0 = [c, d, u];
                self.faces[handle_b.0].0 = [c, v, d];

                // Add our new edge to the auxiliary tables
                let e = Edge::new(c, d);
                self.edges.insert(e);
                self.edge_to_face.insert(e, vec![handle_a, handle_b]);

                // And finally remove the original edge
                self.edges.remove(&edge);
                self.edge_to_face.remove(&edge);
            }
        }
    }
}

impl Edge {
    /// Construct a new edge from the two vertices it connects.
    /// The edge direction will be normalised during construction.
    pub fn new(a: VertexHandle, b: VertexHandle) -> Edge {
        if a > b {
            Edge(b, a)
        } else {
            Edge(a, b)
        }
    }
}

impl Face {
    /// Find the vertex in the face that is not on the provided edge.
    /// Note that if you pass an edge that is not part of this face, the
    /// result will be an arbitrary vertex on this face.
    pub fn vertex_opposite(&self, edge: Edge) -> VertexHandle {
        for &v in self.0.iter() {
            if v != edge.0 && v != edge.1 {
                return v;
            }
        }
        unreachable!();
    }

    fn matches_winding_direction(&self, edge: Edge) -> bool {
        for i in 0..3 {
            let j = (i + 1) % 3;

            if edge.0 == self.0[i] && edge.1 == self.0[j] {
                return true;
            } else if edge.0 == self.0[j] && edge.1 == self.0[i] {
                return false;
            }
        }
        unreachable!();
    }
}

impl Edge {
    /// The start of the edge. Note that edge directions are normalised.
    pub fn start(&self) -> VertexHandle {
        self.0
    }

    /// The end of the edge. Note that edge directions are normalised.
    pub fn end(&self) -> VertexHandle {
        self.1
    }
}

pub struct MeshTopologyBuilder<'a, K: Eq + Hash + Copy, E: Extractor> {
    index_cache: IndexCache<K, VertexHandle>,
    mesh: MeshTopology,
    extractor: &'a mut E,
}

impl<'a, K: Eq + Hash + Copy, E: Extractor> MeshTopologyBuilder<'a, K, E> {
    pub fn new(extractor: &'a mut E) -> Self {
        Self {
            index_cache: IndexCache::new(),
            mesh: MeshTopology::new(),
            extractor,
        }
    }

    pub fn add_vertex(&mut self, key: Option<K>, vertex: Vec3) -> VertexHandle {
        if let Some(index) = key.and_then(|k| self.index_cache.get(k)) {
            index
        } else {
            let index = self.mesh.add_vertex();
            if let Some(key) = key {
                self.index_cache.put(key, index);
            }
            self.extractor.extract_vertex(vertex);
            index
        }
    }

    pub fn add_face(&mut self, a: VertexHandle, b: VertexHandle, c: VertexHandle) {
        self.mesh.add_face(a, b, c);
    }

    pub fn build(self) -> MeshTopology {
        self.mesh
    }
}
