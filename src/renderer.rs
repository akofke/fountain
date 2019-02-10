use crate::scene::Scene;
use crate::camera::Camera;
use crate::Vec3;
use crate::math::to_array;
use crate::geometry::Ray;
use rand::prelude::random;
use std::f32;
use crate::geometry::Object;

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

    pub fn iter_pixels(&self, width: usize, height: usize) -> impl Iterator<Item = [f32; 3]> + '_ {
        const AA_SAMPLES: usize = 128;
        (0..height).rev().flat_map(move |j| (0..width).map(move |i| (i, j)))
            .map(move |(i, j)| {

                let mut color: Vec3 = (0..AA_SAMPLES).map(|_| {
                    let u = (i as f32 + random::<f32>()) / width as f32;
                    let v = (j as f32 + random::<f32>()) / height as f32;

                    let (ray, time) = self.camera.get_ray(u, v);
                    self.cast_ray(&ray, time, 0)
                }).sum();
                color /= AA_SAMPLES as f32;

                color.apply(|x| x.sqrt()); // gamma correction
                to_array(color)
            })
    }

    fn cast_ray(&self, ray: &Ray, time: f32, depth: usize) -> Vec3 {
        if let Some(hit_record) = self.scene.spheres.hit(ray, 0.001, f32::MAX, time) {
//        return (hit_record.normal + Vec3::repeat(1.0)) * 0.5; // normal map
            match hit_record.material.scatter(ray, &hit_record) {
                Some(scatter) if depth < 10 => {
                    scatter.attenuation.component_mul(&self.cast_ray(&scatter.scattered, time, depth + 1))
                },
                _ => Vec3::zeros()
            }
        } else {
            background(&ray.dir)
        }
    }

}