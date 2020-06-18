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

use crate::morton::Morton;
use std::collections::{HashMap, VecDeque};

pub struct LinearHashedOctree<Node> {
    nodes: HashMap<Morton, Node>,
    leaves: Vec<Morton>,
}

impl<Node> LinearHashedOctree<Node> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            leaves: Vec::new(),
        }
    }

    pub fn build<R, C>(&mut self, mut should_refine: R, mut construct_node: C)
    where
        R: FnMut(Morton, &Node) -> bool,
        C: FnMut(Morton) -> Node,
    {
        let mut queue = VecDeque::new();
        queue.push_back(Morton::new());

        while let Some(key) = queue.pop_front() {
            let node = construct_node(key);

            if should_refine(key, &node) {
                for i in 0..8 {
                    queue.push_back(key.child(i));
                }
            } else {
                self.leaves.push(key);
            }

            self.nodes.insert(key, node);
        }
    }

    pub fn walk_leaves<W>(&self, mut walker: W)
    where
        W: FnMut(Morton),
    {
        for &key in &self.leaves {
            walker(key);
        }
    }

    #[inline]
    pub fn get_node(&self, key: &Morton) -> Option<&Node> {
        self.nodes.get(key)
    }
}
