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
use crate::{marching_cubes_tables::EDGE_CONNECTION, morton::Morton};
use std::{cmp::Eq, collections::HashMap, hash::Hash};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct GridKey((usize, usize, usize), (usize, usize, usize));

#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MortonKey(Morton, Morton);

/// Tracks vertex indices to avoid emitting duplicate vertices during marching
/// cubes mesh generation
pub struct IndexCache<K: Eq + Hash, I: Clone> {
    indices: HashMap<K, I>,
}

impl<K: Eq + Hash, I: Clone> IndexCache<K, I> {
    /// Create a new IndexCache
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    /// Put an index in the cache at the given (x, y, z, edge) coordinate
    pub fn put(&mut self, key: K, index: I) {
        self.indices.insert(key, index);
    }

    /// Retrieve an index from the cache at the given (x, y, z, edge) coordinate
    pub fn get(&self, key: K) -> Option<I> {
        self.indices.get(&key).cloned()
    }
}

impl GridKey {
    pub fn new(corners: &[(usize, usize, usize); 8], edge: usize) -> Self {
        let [u, v] = EDGE_CONNECTION[edge];

        let a = corners[u];
        let b = corners[v];

        if a > b {
            Self(b, a)
        } else {
            Self(a, b)
        }
    }
}

impl MortonKey {
    pub fn new(corners: &[Morton; 8], edge: usize) -> Self {
        let [u, v] = EDGE_CONNECTION[edge];
        let (a, b) = (corners[u], corners[v]);

        if a > b {
            Self(b, a)
        } else {
            Self(a, b)
        }
    }
}
