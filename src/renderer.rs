use crate::scene::Scene;
use crate::camera::Camera;
use crate::Vec3;
use crate::math::to_array;
use crate::geom::Ray;
use crate::fast_rand::thread_rng;
use std::f32;
use crate::geom::Object;
use rand::Rng;
use rayon::prelude::*;

pub struct Renderer {
    pub scene: Scene,
    pub camera: Camera,
}

pub fn background(dir: &Vec3) -> Vec3 {
    // scale so t is between 0.0 and 1.0
    let t = 0.5 * (dir[1] + 1.0);
    // linear interpolation based on t
    (1.0 - t) * Vec3::repeat(1.0) + t * Vec3::new(0.5, 0.7, 1.0)
}

impl Renderer {
    pub fn new(scene: Scene, camera: Camera) -> Self {
        Self {
            scene,
            camera
        }
    }

    pub fn render(&self, width: usize, height: usize) -> Vec<Vec3> {
        let mut framebuf: Vec<Vec3> = Vec::with_capacity(width * height);
        framebuf.extend(self.iter_pixels(width, height).map(|p| Vec3::new(p[0], p[1], p[2])));
        framebuf
//        self.iter_pixels(width, height).collect()
    }

    pub fn render_parallel(&self, width: usize, height: usize) -> Vec<Vec3> {
        let mut framebuf: Vec<Vec3> = Vec::with_capacity(width * height);
        framebuf.par_extend(self.iter_pixels_parallel(width, height).map(|p| Vec3::new(p[0], p[1], p[2])));
        framebuf
    }

    pub fn iter_pixels(&self, width: usize, height: usize) -> impl Iterator<Item = [f32; 3]> + '_ {
        (0..height).rev().flat_map(move |j| (0..width).map(move |i| (i, j)))
            .map(move |(i, j)| {
                self.gen_pixel(i, j, width, height)
            })
    }

    pub fn iter_pixels_parallel(&self, width: usize, height: usize) -> impl ParallelIterator<Item = [f32; 3]> + '_ {
        (0..height).into_par_iter().rev().flat_map(move |j| (0..width).into_par_iter().map(move |i| (i, j)))
            .map(move |(i, j)| {
                self.gen_pixel(i, j, width, height)
            })
    }

    fn gen_pixel(&self, i: usize, j: usize, w: usize, h: usize) -> [f32; 3] {
        const AA_SAMPLES: usize = 128;
        let rng = thread_rng(); // TODO: put this in the right place when we have threads

        let mut color: Vec3 = (0..AA_SAMPLES).map(|_| {
            let u = (i as f32 + rng.gen::<f32>()) / w as f32;
            let v = (j as f32 + rng.gen::<f32>()) / h as f32;

            let (ray, time) = self.camera.get_ray(u, v);
            self.cast_ray(ray, time, 0)
        }).sum();
        color /= AA_SAMPLES as f32;

        color.apply(|x| x.sqrt()); // gamma correction
        to_array(color)
    }

    fn cast_ray(&self, ray: Ray, time: f32, depth: usize) -> Vec3 {
        let mut color = Vec3::repeat(1.0);
        let mut ray = ray;
        for _ in 0..10 {
            if let Some(hit_record) = self.scene.spheres.hit(&ray, 0.001, f32::MAX, time) {
                if let Some(scatter) = hit_record.material.scatter(&ray, &hit_record) {
                    color = color.component_mul(&scatter.attenuation);
                    ray = scatter.scattered;
                } else {
                    color = Vec3::zeros();
                    break;
                }
            } else {
                color = color.component_mul(&background(&ray.dir));
                break;
            }
        }
        color
//        if let Some(hit_record) = self.scene.spheres.hit(ray, 0.001, f32::MAX, time) {
//            match hit_record.material.scatter(ray, &hit_record) {
//                Some(scatter) if depth < 10 => {
//                    scatter.attenuation.component_mul(&self.cast_ray(&scatter.scattered, time, depth + 1))
//                },
//                _ => Vec3::zeros()
//            }
//        } else {
//            background(&ray.dir)
//        }
    }

}