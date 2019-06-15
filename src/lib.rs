#![feature(const_generics)]
#![feature(clamp)]
#![feature(const_fn)]

#[macro_use] pub mod macros; // must stay at the top
pub mod geom;
pub mod material;
pub mod camera;
pub mod math;
pub mod fast_rand;
pub mod image;
pub mod scene;
pub mod aabb;
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

pub use geometry::*;
pub use err_float::EFloat;


use nalgebra::{clamp, Point2, Point3, Vector3};
use num::traits::ToPrimitive;
use std::f32;
use num::{Num, Bounded};
use num::traits::NumAssignOps;
use std::fmt::Debug;
use std::any::Any;

pub type Float = f32;

pub type Point2f = Point2<Float>;
pub type Point2i = Point2<i32>;
pub type Point3f = Point3<Float>;
pub type Vec3f = Vector3<Float>;


pub trait Scalar: Num + NumAssignOps + PartialOrd + Bounded + Copy + Debug + Any + From<u8> {
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


pub fn to_rgb(v: Vec3f) -> [u8; 3] {
    let mut arr = [0u8; 3];
    let bytes = v.map(|x| {
        let clamped = clamp(x, 0.0, 1.0) * 255.0;
        clamped.to_u8().unwrap()
    });
    arr.copy_from_slice(bytes.as_slice());
    arr
}

pub fn background(dir: &Vec3f) -> Vec3f {
    // scale so t is between 0.0 and 1.0
    let t = 0.5 * (dir[1] + 1.0);
    // linear interpolation based on t
    (1.0 - t) * Vec3f::repeat(1.0) + t * Vec3f::new(0.5, 0.7, 1.0)
}
