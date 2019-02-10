use crate::geometry::Ray;
use crate::Vec3;
use crate::geometry::HitRecord;
use crate::fast_rand::{random_in_unit_sphere, rand};

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

pub struct Dielectric {
    pub refractive_index: f32
}

fn sclick(cosine: f32, refract_idx: f32) -> f32 {
    let r0 = (1.0 - refract_idx) / (1.0 + refract_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0-cosine).powf(5.0)
}

fn refract(v: &Vec3, n: &Vec3, refract_idx_ratio: f32) -> Option<Vec3> {
    let dt = v.dot(n);
    let discriminant = 1.0 - refract_idx_ratio * refract_idx_ratio * (1.0 - dt*dt);
    if discriminant > 0.0 {
        let refracted = refract_idx_ratio * (v - n*dt) - n*discriminant.sqrt();
        Some(refracted)
    } else { None }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, hit: &HitRecord) -> Option<Scatter> {
        let attenuation = Vec3::repeat(1.0);
        let (outward_normal, refract_idx_ratio, cosine) = if ray_in.dir.dot(&hit.normal) > 0.0{
            (
                -hit.normal,
                self.refractive_index,
                self.refractive_index * ray_in.dir.dot(&hit.normal)
            )
        } else {
            (
                hit.normal,
                1.0 / self.refractive_index,
                -(ray_in.dir.dot(&hit.normal))
            )
        };

        if let Some(refracted) = refract(&ray_in.dir, &outward_normal, refract_idx_ratio) {
            let reflect_prob = sclick(cosine, self.refractive_index);
            if rand::<f32>() < reflect_prob {
                let reflected = reflect(&ray_in.dir, &hit.normal);
                Some(Scatter {
                    attenuation,
                    scattered: Ray {origin: hit.hit, dir: reflected.normalize()}
                })
            } else {
                Some(Scatter {
                    attenuation,
                    scattered: Ray {origin: hit.hit, dir: refracted.normalize()}
                })
            }
        } else {
            let reflected = reflect(&ray_in.dir, &hit.normal);
            Some(Scatter {
                attenuation,
                scattered: Ray {origin: hit.hit, dir: reflected.normalize()}
            })
        }
    }
}