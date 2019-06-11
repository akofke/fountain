#![feature(const_generics)]

#[macro_use] pub mod macros; // must stay at the top
pub mod geom;
pub mod camera;
pub mod material;
pub mod math;
pub mod fast_rand;
pub mod image;
pub mod scene;
pub mod renderer_old;
pub mod aabb;
pub mod bvh;
pub mod morton;
pub mod primitive;
pub mod geometry;
pub mod medium;
pub mod interaction;
pub mod shape;
pub mod renderer;
pub mod integrator;
pub mod spectrum;


use nalgebra::{clamp, Point2, Point3, Vector3};
use crate::geom::Object;
use crate::camera::Camera;
use crate::geom::Ray;
use num::traits::ToPrimitive;
use std::f32;
use crate::scene::Scene;
use crate::geom::Sphere;
use crate::material::*;
use rand::prelude::*;
use crate::camera::Lens;
use num::{Num, Bounded};
use num::traits::NumAssignOps;
use std::fmt::Debug;
use std::any::Any;

pub type Float = f32;

pub type Point2f = Point2<Float>;
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

pub fn cover_example_scene(aspect: f32) -> (Scene, Camera) {

    let mut spheres: Vec<Sphere> = vec![];
    let mut rng = thread_rng();
    spheres.push(Sphere::fixed(vec3f!(0, -1000, 0), 1000.0, Box::new(Lambertian{albedo: vec3f!(0.7, 0.6, 0.5)})));
    for a in -10..10 {
        for b in -10..10 {
            let mat: Box<dyn Material + Sync> = {
                let choose_mat: f32 = random();

                if choose_mat < 0.8 {
                    Box::new(Lambertian {albedo: Vec3f::new(random(), random(), random())})
                } else if choose_mat < 0.95 {
                    Box::new(Metal {albedo: Vec3f::new(rng.gen(), rng.gen(), rng.gen()), fuzz: rng.gen_range(0.0, 0.5)})
                } else {
                    Box::new(Dielectric {refractive_index: 1.5})
                }
            };

            let vel = {
                if rng.gen_ratio(1, 4) {
                    Some(Vec3f::new(rng.gen_range(0.0, 0.2), rng.gen_range(0.0, 0.5), rng.gen_range(0.0, 0.2)))
                } else { None }
            };

            let radius = rng.gen_range(0.15, 0.25);
            let center = Vec3f::new(a as f32 + 0.9 * rng.gen::<f32>(), radius, b as f32 + 0.9 * rng.gen::<f32>());

            spheres.push(Sphere::new(center, radius, mat, vel));
        }
    }

    spheres.push(Sphere::fixed(vec3f!(0, 1, 0), 1.0, Box::new(Dielectric{refractive_index: 1.5})));
    spheres.push(Sphere::fixed(vec3f!(-4, 1, 0), 1.0, Box::new(Lambertian{albedo: vec3f!(0.3, 0.3, 0.8)})));
    spheres.push(Sphere::fixed(vec3f!(4, 1, 0), 1.0, Box::new(Metal{albedo: vec3f!(0.7, 0.6, 0.5), fuzz: 0.01})));
    let scene = Scene::new(spheres);

    let lookfrom = vec3f!(13, 2, 3);
    let lookat = vec3f!(0, 0, 0);
    let lens = Lens {focus_dist: 10.0, aperture: 0.01};
    let dist = 10.0;
    let camera = Camera::new(lookfrom, lookat, vec3f!(0, 1, 0), 20f32.to_radians(), aspect, Some(lens), Some((0.0, 0.5)));

    (scene, camera)
}
