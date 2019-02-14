
pub mod geometry;
pub mod camera;
pub mod material;
#[macro_use]
pub mod math;
pub mod fast_rand;
pub mod image;
pub mod scene;
pub mod renderer;
pub mod aabb;
pub mod bvh;
pub mod morton;
pub mod aac;

pub use crate::math::Vec3;

use nalgebra::clamp;
use crate::geometry::Object;
use crate::camera::Camera;
use crate::geometry::Ray;
use num::traits::ToPrimitive;
use std::f32;
use crate::scene::Scene;
use crate::geometry::Sphere;
use crate::material::*;
use rand::prelude::*;
use crate::camera::Lens;


pub fn to_rgb(v: Vec3) -> [u8; 3] {
    let mut arr = [0u8; 3];
    let bytes = v.map(|x| {
        let clamped = clamp(x, 0.0, 1.0) * 255.0;
        clamped.to_u8().unwrap()
    });
    arr.copy_from_slice(bytes.as_slice());
    arr
}

pub fn background(dir: &Vec3) -> Vec3 {
    // scale so t is between 0.0 and 1.0
    let t = 0.5 * (dir[1] + 1.0);
    // linear interpolation based on t
    (1.0 - t) * Vec3::repeat(1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

pub fn cover_example_scene(aspect: f32) -> (Scene, Camera) {

    let mut spheres: Vec<Sphere> = vec![];
    let mut rng = thread_rng();
    spheres.push(Sphere::fixed(v3!(0, -1000, 0), 1000.0, Box::new(Lambertian{albedo: v3!(0.7, 0.6, 0.5)})));
    for a in -10..10 {
        for b in -10..10 {
            let mat: Box<dyn Material + Sync> = {
                let choose_mat: f32 = random();

                if choose_mat < 0.8 {
                    Box::new(Lambertian {albedo: Vec3::new(random(), random(), random())})
                } else if choose_mat < 0.95 {
                    Box::new(Metal {albedo: Vec3::new(rng.gen(), rng.gen(), rng.gen()), fuzz: rng.gen_range(0.0, 0.5)})
                } else {
                    Box::new(Dielectric {refractive_index: 1.5})
                }
            };

            let vel = {
                if rng.gen_ratio(1, 4) {
                    Some(Vec3::new(rng.gen_range(0.0, 0.2), rng.gen_range(0.0, 0.5), rng.gen_range(0.0, 0.2)))
                } else { None }
            };

            let radius = rng.gen_range(0.15, 0.25);
            let center = Vec3::new(a as f32 + 0.9 * rng.gen::<f32>(), radius, b as f32 + 0.9 * rng.gen::<f32>());

            spheres.push(Sphere::new(center, radius, mat, vel));
        }
    }

    spheres.push(Sphere::fixed(v3!(0, 1, 0), 1.0, Box::new(Dielectric{refractive_index: 1.5})));
    spheres.push(Sphere::fixed(v3!(-4, 1, 0), 1.0, Box::new(Lambertian{albedo: v3!(0.3, 0.3, 0.8)})));
    spheres.push(Sphere::fixed(v3!(4, 1, 0), 1.0, Box::new(Metal{albedo: v3!(0.7, 0.6, 0.5), fuzz: 0.01})));
    let scene = Scene::new(spheres);

    let lookfrom = v3!(13, 2, 3);
    let lookat = v3!(0, 0, 0);
    let lens = Lens {focus_dist: 10.0, aperture: 0.01};
    let dist = 10.0;
    let camera = Camera::new(lookfrom, lookat, v3!(0, 1, 0), 20f32.to_radians(), aspect, Some(lens), Some((0.0, 0.5)));

    (scene, camera)
}
