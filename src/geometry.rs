use nalgebra::Vector3;

pub type Vec3 = Vector3<f32>;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

pub struct HitRecord {
    pub dist: f32,
    pub hit: Vec3,
    pub normal: Vec3
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Sphere {center, radius}
    }

    pub fn ray_intersect(&self, orig: &Vec3, dir: &Vec3) -> Option<f32> {
        // get vector from origin to center of sphere
        let oc = self.center - *orig;
        let tca = oc.dot(dir);
        let d2 = oc.norm_squared() - tca * tca;
        if d2 > self.radius * self.radius { return None; }
        let thc = (self.radius * self.radius - d2).sqrt();
        let t0 = tca - thc;
        if t0 > 0.0  { Some(t0) } else { None }
    }
}
