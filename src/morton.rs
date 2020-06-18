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

use crate::math::Vec3;
use std;

const THREE_2: usize = 9;
const THREE_1: usize = 3;
const THREE_0: usize = 1;
const SHIFT_2A: u64 = 18;
const SHIFT_2B: u64 = 36;
const SHIFT_1A: u64 = 6;
const SHIFT_1B: u64 = 12;
const SHIFT_0A: u64 = 2;
const SHIFT_0B: u64 = 4;
const DILATE_MASK_2: u64 = 0x7FC0_000F_F800_01FF; // 0-9, 27-36, 54-63
const DILATE_MASK_1: u64 = 0x01C0_E070_381C_0E07; // 0-3, 9-12, 18-21
const DILATE_MASK_0: u64 = 0x9249_2492_4924_9249; // 0,3,6,9,12
const DILATE_TZ: u64 = 0x4924_9249_2492_4924; // 2,5,8,11,14
const DILATE_TY: u64 = 0x2492_4924_9249_2492; // 1,4,7,10,13
const DILATE_TX: u64 = 0x9249_2492_4924_9249; // 0,3,6,9,12
const DILATE_T1: u64 = 0xB6DB_6DB6_DB6D_B6DB; // ~tz
const DILATE_T2: u64 = 0xDB6D_B6DB_6DB6_DB6D; // ~ty
const DILATE_T3: u64 = 0x6DB6_DB6D_B6DB_6DB6; // ~tx
const LG2_3: f64 = 0.480_898_346_96; // 1.0 / (ln(2) * 3);
const MAX_LEVEL: usize = (8 * 8 - 1) / 3; // ((sizeof(u64) in bits) - 1) / 3

/// Refer to an octree node via interleaved integer coordinates
#[derive(Default, Hash, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Morton(u64);

impl Morton {
    /// Creates a morton code that points to the root octree node
    pub fn new() -> Self {
        Morton(1)
    }

    /// Creates a morton code with a specific key code
    pub fn with_key(key: u64) -> Self {
        Morton(key)
    }

    /// The depth of this octree node
    pub fn level(&self) -> usize {
        match self.0 {
            0 => 0,
            a => ((a as f64).ln() * LG2_3).floor() as usize,
        }
    }

    /// The parent node to this octree node.
    pub fn parent(&self) -> Self {
        Morton((self.0 >> 3).max(1))
    }

    /// Get one of the 8 child nodes of this octree node.
    pub fn child(&self, which: u8) -> Self {
        Morton((self.0 << 3) | u64::from(which))
    }

    /// The distance from the center of the octree node to the edge (i.e. half the width/height/depth).
    pub fn size(&self) -> f32 {
        1.0 / ((2 << self.level()) as f32)
    }

    /// Get the center of this octree node as a vector.
    pub fn center(&self) -> Vec3 {
        let mut bz = (self.0 >> 2) & DILATE_MASK_0;
        let mut by = (self.0 >> 1) & DILATE_MASK_0;
        let mut bx = self.0 & DILATE_MASK_0;

        let level = self.level();

        if level > THREE_0 {
            bz = (bz | (bz >> SHIFT_0A) | (bz >> SHIFT_0B)) & DILATE_MASK_1;
            by = (by | (by >> SHIFT_0A) | (by >> SHIFT_0B)) & DILATE_MASK_1;
            bx = (bx | (bx >> SHIFT_0A) | (bx >> SHIFT_0B)) & DILATE_MASK_1;

            if level > THREE_1 {
                bz = (bz | (bz >> SHIFT_1A) | (bz >> SHIFT_1B)) & DILATE_MASK_2;
                by = (by | (by >> SHIFT_1A) | (by >> SHIFT_1B)) & DILATE_MASK_2;
                bx = (bx | (bx >> SHIFT_1A) | (bx >> SHIFT_1B)) & DILATE_MASK_2;

                if level > THREE_2 {
                    bz = bz | (bz >> SHIFT_2A) | (bz >> SHIFT_2B);
                    by = by | (by >> SHIFT_2A) | (by >> SHIFT_2B);
                    bx = bx | (bx >> SHIFT_2A) | (bx >> SHIFT_2B);
                }
            }
        }

        let length_mask = (1 << level) - 1;
        bz &= length_mask;
        by &= length_mask;
        bx &= length_mask;

        let size = self.size();
        let size2 = size * 2.0;

        Vec3::new(
            (bx as f32) * size2 + size,
            (by as f32) * size2 + size,
            (bz as f32) * size2 + size,
        )
    }

    /// Assuming that self is a point on the dual mesh, finds the 8 corresponding vertices on the primal mesh.
    pub fn primal_vertex(&self, level: usize, which: usize) -> Morton {
        let k = 1 << (3 * level);
        let k_plus_one = k << 1;

        let vk = *self + Morton(which as u64);
        let dk = (vk - Morton(k)).0;

        if vk.0 >= k_plus_one
            || (dk & DILATE_TX) == 0
            || (dk & DILATE_TY) == 0
            || (dk & DILATE_TZ) == 0
        {
            Morton(0)
        } else {
            Morton(vk.0 << (3 * (MAX_LEVEL - level)))
        }
    }

    /// Assuming that self is a point on the primal mesh, finds the 8 corresponding vertices on the dual mesh.
    pub fn dual_vertex(&self, level: usize, which: usize) -> Morton {
        let dk = Morton(self.0 >> (3 * (MAX_LEVEL - level)));

        dk - Morton(which as u64)
    }
}

impl std::ops::Add for Morton {
    type Output = Morton;

    fn add(self, other: Morton) -> Morton {
        Morton(
            (((self.0 | DILATE_T1) + (other.0 & DILATE_TZ)) & DILATE_TZ)
                | (((self.0 | DILATE_T2) + (other.0 & DILATE_TY)) & DILATE_TY)
                | (((self.0 | DILATE_T3) + (other.0 & DILATE_TX)) & DILATE_TX),
        )
    }
}

impl std::ops::Sub for Morton {
    type Output = Morton;

    fn sub(self, other: Morton) -> Morton {
        Morton(
            (((self.0 & DILATE_TZ).wrapping_sub(other.0 & DILATE_TZ)) & DILATE_TZ)
                | (((self.0 & DILATE_TY).wrapping_sub(other.0 & DILATE_TY)) & DILATE_TY)
                | (((self.0 & DILATE_TX).wrapping_sub(other.0 & DILATE_TX)) & DILATE_TX),
        )
    }
}

impl std::convert::From<Morton> for usize {
    fn from(src: Morton) -> usize {
        src.0 as usize
    }
}

impl std::fmt::Debug for Morton {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}
