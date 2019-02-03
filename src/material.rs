use crate::geometry::Ray;
use crate::geometry::Vec3;
use crate::geometry::HitRecord;
use crate::random_in_unit_sphere;

#[derive(Clone, Copy)]
pub struct Scatter {
    pub attenuation: Vec3,
    pub scattered: Ray
}

pub trait Material {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<Scatter>;
}

pub struct Lambertian {
    pub albedo: Vec3
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let target = hit.normal + random_in_unit_sphere();
        let scattered = Ray {origin: hit.hit, dir: target.normalize()};
        Some(Scatter{attenuation: self.albedo, scattered})
    }
}

pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f32
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let reflected = reflect(&ray_in.dir, &hit.normal);
        let scattered = Ray {origin: hit.hit, dir: (reflected + self.fuzz*random_in_unit_sphere()).normalize()};
        if scattered.dir.dot(&hit.normal) > 0.0 {
            Some(Scatter {attenuation: self.albedo, scattered})
        } else {
            None
        }
    }
}