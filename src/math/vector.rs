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

///! Ideally we'd reuse an exiting geometry library, but in the interest both
/// of minimising dependencies, and of compatibility with multiple geometry
/// libraries, we'll define our own.
use std;

/// A 2 dimensional vector
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// A 3 dimensional vector
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

// Plus and multiply operators can't be used as separators in macro repetition,
// so I use a fold operation instead
macro_rules! fold {
    ($op:tt, $x:expr, $y:expr) => {
        $x $op $y
    };
    ($op:tt, $x:expr, $y:expr, $($rest:expr),+) => {
        fold!($op, ($x $op $y), $($rest),*)
    }
}

macro_rules! impl_arithmetic_op {
    ($name:ident, $op_name:ident, $op_small_name:ident, $op:tt { $($field:ident ),+ }) => {
        impl std::ops::$op_name for $name {
            type Output = $name;
            fn $op_small_name(self, other: $name) -> $name {
                $name::new($(self.$field $op other.$field),*)
            }
        }
        impl std::ops::$op_name for &$name {
            type Output = $name;
            fn $op_small_name(self, other: &$name) -> $name {
                $name::new($(self.$field $op other.$field),*)
            }
        }
        impl std::ops::$op_name<&$name> for $name {
            type Output = $name;
            fn $op_small_name(self, other: &$name) -> $name {
                $name::new($(self.$field $op other.$field),*)
            }
        }
        impl std::ops::$op_name<f32> for $name {
            type Output = $name;
            fn $op_small_name(self, other: f32) -> $name {
                $name::new($(self.$field $op other),*)
            }
        }
        impl std::ops::$op_name<$name> for f32 {
            type Output = $name;
            fn $op_small_name(self, other: $name) -> $name {
                $name::new($(self $op other.$field),*)
            }
        }
    };
}
macro_rules! impl_arithmetic_assign_op {
    ($name:ident, $op_assign_name:ident, $op_assign_small_name:ident, $op:tt { $($field:ident ),+ }) => {
        impl std::ops::$op_assign_name for $name {
            fn $op_assign_small_name(&mut self, other: $name) {
                $(self.$field $op other.$field);*
            }
        }
        impl std::ops::$op_assign_name<f32> for $name {
            fn $op_assign_small_name(&mut self, other: f32) {
                $(self.$field $op other);*
            }
        }
    };
}

macro_rules! impl_vector {
    ($name:ident { $($field:ident ),+ }) => {
        impl $name {
            /// Create a vector
            pub fn new($($field : f32),*) -> Self {
                Self { $($field),* }
            }
            /// Create a vector by repeating a single value
            pub fn from_scalar(f: f32) -> Self {
                Self { $($field: f),* }
            }

            /// Create a vector with all coordinates set to zero
            pub fn zero() -> Self {
                Self{ $($field: 0.0),*}
            }
            /// Create a vector with all coordinates set to one
            pub fn one() -> Self {
                Self{ $($field: 1.0),*}
            }

            /// Squared Euclidean length of this vector
            pub fn len_sq(&self) -> f32 {
                fold!(+, $(self.$field * self.$field),*)
            }

            /// Euclidean length of this vector
            pub fn len(&self) -> f32 {
                self.len_sq().sqrt()
            }

            /// Normalised copy of this vector
            pub fn normalised(&self) -> Option<Self> {
                let l = self.len();
                if l.abs() < std::f32::EPSILON {
                    None
                } else {
                    Some(Self {
                        $($field: self.$field / l),*
                    })
                }
            }

            /// Create a new vector by applying the provided function to each component in this vector
            pub fn map<F: Fn(f32) -> f32>(&self, f: F) -> Self {
                $name::new( $(f(self.$field)),* )
            }
            /// Test if any component matches a predicate
            pub fn any<F: Fn(f32) -> bool>(&self, f: F) -> bool {
                fold!(||, $(f(self.$field)),* )
            }
            /// Test if every component matches a predicate
            pub fn all<F: Fn(f32) -> bool>(&self, f: F) -> bool {
                fold!(&&, $(f(self.$field)),* )
            }
        }

        impl std::default::Default for $name {
            fn default() -> Self {
                Self::zero()
            }
        }

        impl std::ops::Neg for $name {
            type Output = $name;
            fn neg(self) -> $name {
                $name::new($(-self.$field),*)
            }
        }

        impl std::iter::Sum for $name {
            fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), std::ops::Add::add)
            }
        }
        impl<'a> std::iter::Sum<&'a $name> for $name {
            fn sum<I: Iterator<Item=&'a Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), std::ops::Add::add)
            }
        }

        impl std::iter::Product for $name {
            fn product<I: Iterator<Item=Self>>(iter: I) -> Self {
                iter.fold(Self::one(), std::ops::Mul::mul)
            }
        }
        impl<'a> std::iter::Product<&'a $name> for $name {
            fn product<I: Iterator<Item=&'a Self>>(iter: I) -> Self {
                iter.fold(Self::one(), std::ops::Mul::mul)
            }
        }

        impl std::ops::Index<usize> for $name {
            type Output = f32;
            fn index(&self, index: usize) -> &f32 {
                [$(&self.$field),*][index]
            }
        }

        impl std::ops::IndexMut<usize> for $name {
            fn index_mut(&mut self, index: usize) -> &mut f32 {
                [$(&mut self.$field),*][index]
            }
        }

        impl_arithmetic_op!($name, Add, add, + { $($field),* });
        impl_arithmetic_op!($name, Sub, sub, - { $($field),* });
        impl_arithmetic_op!($name, Mul, mul, * { $($field),* });
        impl_arithmetic_op!($name, Div, div, / { $($field),* });
        impl_arithmetic_assign_op!($name, AddAssign, add_assign, += { $($field),* });
        impl_arithmetic_assign_op!($name, SubAssign, sub_assign, -= { $($field),* });
        impl_arithmetic_assign_op!($name, MulAssign, mul_assign, *= { $($field),* });
        impl_arithmetic_assign_op!($name, DivAssign, div_assign, /= { $($field),* });
    };
}

impl_vector!(Vec2 { x, y });
impl_vector!(Vec3 { x, y, z });

impl Vec2 {
    pub fn extend(&self, z: f32) -> Vec3 {
        vec3(self.x, self.y, z)
    }
}

impl Vec3 {
    /// Create a vector by taking the absolute value of each component in this
    /// vector
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    /// Sum all of the components in this vector
    pub fn component_sum(&self) -> f32 {
        self.x + self.y + self.z
    }

    /// Find the maximum value out of all components in this vector
    pub fn max_component(&self) -> f32 {
        self.x.max(self.y.max(self.z))
    }

    /// Find the minimum value out of all components in this vector
    pub fn min_component(&self) -> f32 {
        self.x.min(self.y.min(self.z))
    }

    /// Find the index of the maximum value out of all components in this vector
    pub fn max_component_index(&self) -> usize {
        if self.x > self.y && self.x > self.z {
            0
        } else if self.y > self.z {
            1
        } else {
            2
        }
    }

    /// Find the index of the minimum value out of all components in this vector
    pub fn min_component_index(&self) -> usize {
        if self.x < self.y && self.x < self.z {
            0
        } else if self.y < self.z {
            1
        } else {
            2
        }
    }

    /// Take only the component in this vector that lies along the closest
    /// cardinal axis
    pub fn clamp_to_cardinal_axis(&self) -> Vec3 {
        let p = self.abs();
        if p.x > p.y && p.x > p.z {
            Vec3::new(self.x, 0.0, 0.0)
        } else if p.y > p.z {
            Vec3::new(0.0, self.y, 0.0)
        } else {
            Vec3::new(0.0, 0.0, self.z)
        }
    }

    /// Calculate the dot product of this vector and another
    pub fn dot(&self, other: Self) -> f32 {
        (*self * other).component_sum()
    }

    /// Compute the cross product of this vector and another
    pub fn cross(&self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Create a vector by taking the min value of each component in this vector
    /// and another
    pub fn min(&self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    /// Create a vector by taking the max value of each component in this vector
    /// and another
    pub fn max(&self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    /// Create a vector by linearly interpolating between this vector and
    /// another
    pub fn lerp(&self, other: Self, f: f32) -> Self {
        let of = 1.0 - f;

        Self {
            x: of * self.x + f * other.x,
            y: of * self.y + f * other.y,
            z: of * self.z + f * other.z,
        }
    }

    pub fn yz(&self) -> Vec2 {
        vec2(self.y, self.z)
    }

    pub fn xz(&self) -> Vec2 {
        vec2(self.x, self.z)
    }

    pub fn xy(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_works() {
        assert_eq!(vec2(1.0, 2.0).len_sq(), 5.0);
        assert_eq!(vec3(1.0, 2.0, 3.0).len_sq(), 14.0);
    }
}
