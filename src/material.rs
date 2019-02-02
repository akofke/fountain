use crate::geometry::Ray;
use crate::geometry::Vec3;
use crate::geometry::HitRecord;

pub struct Scatter {
    pub attenuation: Vec3,
    pub scattered: Ray
}

pub trait Material {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<Scatter>;
}