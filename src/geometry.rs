use nalgebra::{Vector3};
use std::ops::Deref;
use crate::material::Material;

pub type Vec3 = Vector3<f32>;


#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn at_param(&self, t: f32) -> Vec3 {
        self.origin + (self.dir * t)
    }
}

#[derive(Copy, Clone)]
pub struct HitRecord<'a> {
    pub dist: f32,
    pub hit: Vec3,
    pub normal: Vec3,
    pub material: &'a dyn Material
}

pub trait Object {
    // lifetimes: 'a lives at least as long as 'b
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'b>>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Box<dyn Material>
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Box<dyn Material>) -> Self {
        Sphere {center, radius, material}
    }

//    pub fn ray_intersect(&self, orig: &Vec3, dir: &Vec3) -> Option<f32> {
//        // get vector from origin to center of sphere
//        let oc = self.center - *orig;
//        let tca = oc.dot(dir);
//        let d2 = oc.norm_squared() - tca * tca;
//        if d2 > self.radius * self.radius { return None; }
//        let thc = (self.radius * self.radius - d2).sqrt();
//        let t0 = tca - thc;
//        if t0 > 0.0  { Some(t0) } else { None }
//    }
}

impl Object for Sphere {
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'b>> {
        let oc = ray.origin - self.center;
        let a = ray.dir.norm_squared();
        let b = oc.dot(&ray.dir);
        let c = oc.norm_squared() - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            for &t in &[(-b - discriminant.sqrt()) / a, (-b + discriminant.sqrt()) / a] {
                if t < t_max && t > t_min {
                    return Some(HitRecord {
                        dist: t,
                        hit: ray.at_param(t),
                        normal: (ray.at_param(t) - self.center) / self.radius,

                        // return a reference to the owned Material trait object.
                        // The HitRecord has a lifetime of 'b which is <= a, the lifetime
                        // of the sphere struct (&self?)
                        material: self.material.as_ref()
                    });
                }
            }
        }
        None
    }
}

// Implementation for homogeneous collection of objects
// Don't know exactly what Deref will cover but it works for Vec
// lifetimes: O: 'static means if O contains any references (which Objects probably won't),
// they have to have static lifetime
impl<T, O: Object + 'static> Object for T where T: Deref<Target = [O]>
{
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord<'b>> {
        let mut hit_record: Option<HitRecord> = None;
        let mut closest_so_far = t_max;
        for obj in self.iter() {
            if let Some(hit) = obj.hit(ray, t_min, closest_so_far) {
                hit_record = Some(hit);
                closest_so_far = hit.dist;
            }
        }
        hit_record
    }
}

// Fallback using dynamic dispatch on a collection of heterogeneous scene objects
// Should use specialized SoA collections e.g. SpheresList where possible
//impl Object for &[Box<dyn Object>] {
//    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
//        let mut hit_record: Option<HitRecord> = None;
//        let mut closest_so_far = t_max;
//        for obj in self.iter() {
//            if let Some(hit) = obj.hit(ray, t_min, closest_so_far) {
//                hit_record = Some(hit);
//                closest_so_far = hit.dist;
//            }
//        }
//        hit_record
//    }
//}
