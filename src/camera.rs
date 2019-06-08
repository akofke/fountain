use crate::Vec3;
use crate::geom::Ray;
use nalgebra::Vector2;
use rand::prelude::*;
use rand::distributions::Uniform;
use crate::fast_rand::random_in_unit_disk;

pub struct Camera {
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    origin: Vec3,
    lens_radius: f32,
    orientation: (Vec3, Vec3, Vec3),
    time_distribution: Option<Uniform<f32>>
}

pub struct Lens {
    pub aperture: f32,
    pub focus_dist: f32
}


impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, up: Vec3, vfov: f32, aspect: f32, lens: Option<Lens>, time_delta: Option<(f32, f32)>) -> Camera {
        let lens = lens.unwrap_or(Lens {aperture: 0.0, focus_dist: 1.0});
        let half_height = f32::tan(vfov / 2.0);
        let half_width = aspect * half_height;
        let w = (lookfrom - lookat).normalize(); // "z" vector, points away from scene
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);
        let lower_left_corner = lookfrom - (half_width * lens.focus_dist* u) - (half_height * lens.focus_dist * v) - lens.focus_dist * w;
        let horizontal = 2.0 * half_width * lens.focus_dist * u;
        let vertical = 2.0 * half_height * lens.focus_dist * v;

        Camera {
            lower_left_corner,
            horizontal,
            vertical,
            origin: lookfrom,
            lens_radius: lens.aperture / 2.0,
            orientation: (u, v, w),
            time_distribution: time_delta.map(|t| Uniform::new_inclusive(t.0, t.1))
        }
    }

    pub fn with_aspect(aspect: f32) -> Camera {
        Camera::new(
            Vec3::zeros(),
            Vec3::new(0., 0., -1.),
            Vec3::new(0., 1., 0.),
            90f32.to_radians(),
            aspect,
            None,
            None
        )
    }

    pub fn get_ray(&self, x: f32, y: f32) -> (Ray, f32) {
        let rd = self.lens_radius * random_in_unit_disk();
        let (u, v, _) = self.orientation;
        let offset = u * rd[0] + v * rd[1];
        let time = self.time_distribution.map_or(0.0, |dist| dist.sample(&mut thread_rng()));
        let r = Ray {
            origin: self.origin + offset,
            dir: (self.lower_left_corner + x*self.horizontal + y*self.vertical - self.origin - offset).normalize()
        };
        (r, time)
    }
}
