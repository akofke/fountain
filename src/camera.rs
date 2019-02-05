use crate::geometry::Vec3;
use crate::geometry::Ray;

pub struct Camera {
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    origin: Vec3
}

impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, up: Vec3, vfov: f32, aspect: f32) -> Camera {
        let half_height = f32::tan(vfov / 2.0);
        let half_width = aspect * half_height;
        let w = (lookfrom - lookat).normalize(); // "z" vector, points away from scene
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);
        let lower_left_corner = lookfrom - half_width*u - half_height*v - w;
        let horizontal = 2.0 * half_width * u;
        let vertical = 2.0 * half_height * v;

        Camera {
            lower_left_corner,
            horizontal,
            vertical,
            origin: lookfrom
        }
    }

    pub fn with_aspect(aspect: f32) -> Camera {
        Camera::new(
            Vec3::zeros(),
            Vec3::new(0., 0., -1.),
            Vec3::new(0., 1., 0.),
            90f32.to_radians(),
            aspect
        )
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            dir: (self.lower_left_corner + u*self.horizontal + v*self.vertical - self.origin).normalize()
        }
    }
}
