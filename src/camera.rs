use crate::Vec3;
use crate::geometry::Ray;
use nalgebra::Vector2;
use rand::prelude::random;

pub struct Camera {
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    origin: Vec3,
    lens_radius: f32,
    orientation: (Vec3, Vec3, Vec3)
}

pub struct Lens {
    pub aperture: f32,
    pub focus_dist: f32
}

fn random_in_unit_disk() -> Vector2<f32> {
    loop {
        let p: Vector2<f32> = 2.0 * Vector2::<f32>::new(random(), random()) - Vector2::repeat(1.0f32);
        if p.norm_squared() < 1.0 { break p }
    }
}

impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, up: Vec3, vfov: f32, aspect: f32, lens: Option<Lens>) -> Camera {
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
            orientation: (u, v, w)
        }
    }

    pub fn with_aspect(aspect: f32) -> Camera {
        Camera::new(
            Vec3::zeros(),
            Vec3::new(0., 0., -1.),
            Vec3::new(0., 1., 0.),
            90f32.to_radians(),
            aspect,
            None
        )
    }

    pub fn get_ray(&self, x: f32, y: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let (u, v, _) = self.orientation;
        let offset = u * rd[0] + v * rd[1];
        Ray {
            origin: self.origin + offset,
            dir: (self.lower_left_corner + x*self.horizontal + y*self.vertical - self.origin - offset).normalize()
        }
    }
}
