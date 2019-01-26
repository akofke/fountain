use crate::math::Vector3f;

pub struct Sphere {
    pub center: Vector3f,
    pub radius: f32,
}

pub struct HitRecord {
    pub dist: f32,
    pub hit: Vector3f,
    pub normal: Vector3f
}

impl Sphere {
    pub fn new(center: Vector3f, radius: f32) -> Self {
        Sphere {center, radius}
    }

    pub fn ray_intersect(&self, orig: &Vector3f, dir: &Vector3f) -> Option<f32> {
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
