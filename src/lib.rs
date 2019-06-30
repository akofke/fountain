#![feature(const_generics)]
#![feature(clamp)]
#![feature(const_fn)]

#[macro_use] pub mod macros; // must stay at the top
pub mod camera;
pub mod math;
pub mod fast_rand;
pub mod image;
pub mod scene;
pub mod bvh;
pub mod morton;
pub mod primitive;
pub mod geometry;
pub mod medium;
pub mod interaction;
pub mod shapes;
pub mod renderer;
pub mod integrator;
pub mod spectrum;
pub mod err_float;
pub mod film;
pub mod filter;
pub mod sampler;
pub mod reflection;
pub mod fresnel;
pub mod material;

pub use geometry::*;
pub use math::*;
pub use err_float::EFloat;


use cgmath::{Vector2, Point2, Vector3, Point3};
use std::f32;
use num::{Num, Bounded, Signed, NumCast};
use num::traits::NumAssignOps;
use std::fmt::Debug;
use std::any::Any;

pub type Float = f32;

pub type Point2f = Point2<Float>;
pub type Point2i = Point2<i32>;
pub type Point3f = Point3<Float>;
pub type Vec3f = Vector3<Float>;
pub type Vec2f = Vector2<Float>;
pub type Vec2i = Vector2<i32>;


pub trait Scalar: Num + NumAssignOps + NumCast + PartialOrd + Bounded + Copy + Debug + Any + From<u8> {
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

// Can't do this because of conflicting implementations...

//impl<S> Scalar for S
//    where S: num::PrimInt
//{
//    fn min(self, other: Self) -> Self {
//        Ord::min(self, other)
//    }
//
//    fn max(self, other: Self) -> Self {
//        Ord::max(self, other)
//    }
//}

impl Scalar for f32 {
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}

impl Scalar for f64 {
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}

// TODO: others...
impl Scalar for u32 {
    fn min(self, other: Self) -> Self {
        Ord::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }
}

impl Scalar for i32 {
    fn min(self, other: Self) -> Self {
        Ord::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }
}

pub trait ElementAbs {
    fn abs(&self) -> Self;
}

impl<S: Signed + Copy> ElementAbs for cgmath::Vector3<S> {
    fn abs(&self) -> Self {
        self.map(|v| v.abs())
    }
}

pub trait ComponentWiseExt {
    fn abs(self) -> Self;

//    fn ceil(self) -> Self;
//
//    fn floor(self) -> Self;

    fn min(self, other: Self) -> Self;

    fn max(self, other: Self) -> Self;
}

impl ComponentWiseExt for cgmath::Vector3<Float> {
    fn abs(self) -> Self {
        self.map(|v| v.abs())
    }

    fn min(self, other: Self) -> Self {
        Vector3::new(
            Float::min(self.x, other.x),
            Float::min(self.y, other.y),
            Float::min(self.z, other.z)
        )
    }

    fn max(self, other: Self) -> Self {
        Vector3::new(
            Float::max(self.x, other.x),
            Float::max(self.y, other.y),
            Float::max(self.z, other.z)
        )
    }
}

impl<S> ComponentWiseExt for cgmath::Point2<S>
where S: Copy + Signed + Ord
{
    fn abs(self) -> Self {
        self.map(|v| v.abs())
    }

    fn min(self, other: Self) -> Self {
        Point2::new(
            S::min(self.x, other.x),
            S::min(self.y, other.y),
        )
    }

    fn max(self, other: Self) -> Self {
        Point2::new(
            S::max(self.x, other.x),
            S::max(self.y, other.y),
        )
    }
}


pub fn background(dir: &Vec3f) -> Vec3f {
    // scale so t is between 0.0 and 1.0
    let t = 0.5 * (dir[1] + 1.0);
    // linear interpolation based on t
    (1.0 - t) * Vec3f::new(1.0, 1.0, 1.0) + t * Vec3f::new(0.5, 0.7, 1.0)
}
