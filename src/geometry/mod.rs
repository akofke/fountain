use crate::Vec3;
use nalgebra::Transform3;
use nalgebra::Point3;

pub mod bounds;

pub struct Ray {
    pub origin: Point3<f32>,
    pub dir: Vec3,
    pub t_max: f32,
    pub time: f32,

    // TODO: medium, differentials
}

impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + (self.dir * t)
    }
}


pub struct Normal3 {
    pub v: Vec3
}

impl Normal3 {
}

pub struct Transform {
    pub t: Transform3<f32>,
    pub invt: Transform3<f32>
}

impl Transform {
    pub fn transform_normal(&self, n: &Normal3) -> Normal3 {
        let n = n.v;
        // transform by the transpose of the inverse
        let x = self.invt[(0, 0)]*n.x + self.invt[(1, 0)]*n.y + self.invt[(2, 0)]*n.z;
        let y = self.invt[(0, 1)]*n.x + self.invt[(1, 1)]*n.y + self.invt[(2, 1)]*n.z;
        let z = self.invt[(0, 2)]*n.x + self.invt[(1, 2)]*n.y + self.invt[(2, 2)]*n.z;
        Normal3 { v: v3!(x, y, z)}


    }
}

