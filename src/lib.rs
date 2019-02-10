
pub mod geometry;
pub mod camera;
pub mod material;
#[macro_use]
pub mod math;
pub mod random;
pub mod image;
pub mod scene;
pub mod renderer;

pub use crate::math::Vec3;

use nalgebra::clamp;
use crate::geometry::Object;
use crate::camera::Camera;
use rand::random;
use crate::geometry::Ray;
use num::traits::ToPrimitive;
use std::f32;


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

//pub fn render(width: usize, height: usize, scene: impl Object, camera: &Camera) -> Vec<Vec3> {
//    const AA_SAMPLES: usize = 128;
//    let mut framebuf: Vec<Vec3> = Vec::with_capacity(width * height);
//
//    for j in (0..height).rev() {
//        for i in 0..width {
//
//            let mut color: Vec3 = (0..AA_SAMPLES).map(|_| {
//                let u = (i as f32 + random::<f32>()) / width as f32;
//                let v = (j as f32 + random::<f32>()) / height as f32;
//
//                let (ray, time) = camera.get_ray(u, v);
//                cast_ray(&ray, time, &scene, 0)
//            }).sum();
//            color /= AA_SAMPLES as f32;
//
//            color.apply(|x| x.sqrt()); // gamma correction
//            framebuf.push(color);
//        }
//    }
//    return framebuf;
//}
//
//pub fn cast_ray(ray: &Ray, time: f32, scene: &impl Object, depth: usize) -> Vec3 {
//    if let Some(hit_record) = scene.hit(ray, 0.001, f32::MAX, time) {
////        return (hit_record.normal + Vec3::repeat(1.0)) * 0.5; // normal map
//        match hit_record.material.scatter(ray, &hit_record) {
//            Some(scatter) if depth < 10 => {
//                scatter.attenuation.component_mul(&cast_ray(&scatter.scattered, time, scene, depth + 1))
//            },
//            _ => Vec3::zeros()
//        }
//    } else {
//        background(&ray.dir)
//    }
//}

