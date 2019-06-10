use crate::Vec3f;
use std::ops::Deref;
use crate::material::Material;



#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Vec3f,
    pub dir: Vec3f,
}

impl Ray {
    pub fn at_param(&self, t: f32) -> Vec3f {
        self.origin + (self.dir * t)
    }
}

#[derive(Copy, Clone)]
pub struct HitRecord<'a> {
    pub dist: f32,
    pub hit: Vec3f,
    pub normal: Vec3f,
    pub material: &'a dyn Material
}

pub trait Object {
    // lifetimes: 'a lives at least as long as 'b
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32, time: f32) -> Option<HitRecord<'b>>;
}

pub struct Sphere {
    pub center: Vec3f,
    pub radius: f32,
    pub material: Box<dyn Material + Sync>,
    pub velocity: Option<Vec3f>
}

impl Sphere {

    pub fn new(center: Vec3f, radius: f32, material: Box<dyn Material + Sync>, velocity: Option<Vec3f>) -> Self {
        Sphere {center, radius, material, velocity}
    }


    pub fn fixed(center: Vec3f, radius: f32, material: Box<dyn Material + Sync>) -> Self {
        Sphere {center, radius, material, velocity: None}
    }

    fn center(&self, time: f32) -> Vec3f {
        self.velocity.map_or(self.center, |v| {
            self.center + time*v
        })
    }

}

impl Object for Sphere {
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32, time: f32) -> Option<HitRecord<'b>> {
        let center = self.center(time);
        let oc = ray.origin - center;
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
                        normal: (ray.at_param(t) - center) / self.radius,

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
    fn hit<'a: 'b, 'b>(&'a self, ray: &Ray, t_min: f32, t_max: f32, time: f32) -> Option<HitRecord<'b>> {
        let mut hit_record: Option<HitRecord> = None;
        let mut closest_so_far = t_max;
        for obj in self.iter() {
            if let Some(hit) = obj.hit(ray, t_min, closest_so_far, time) {
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
