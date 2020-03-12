use std::ops::Deref;

use cgmath::InnerSpace;

pub use bounds::*;
pub use transform::*;

use crate::{Point3f, Vec3f};
use crate::err_float::{next_float_down, next_float_up};
use crate::Float;

pub mod bounds;
pub mod transform;

pub fn distance(p1: Point3f, p2: Point3f) -> Float {
    (p1 - p2).magnitude()
}

pub fn distance_sq(p1: Point3f, p2: Point3f) -> Float {
    (p1 - p2).magnitude2()
}

// TODO: make generic?
pub fn permute_point(p: Point3f, ix: usize, iy: usize, iz: usize) -> Point3f {
    Point3f::new(p[ix], p[iy], p[iz])
}

pub fn permute_vec(v: Vec3f, ix: usize, iy: usize, iz: usize) -> Vec3f {
    Vec3f::new(v[ix], v[iy], v[iz])
}

pub fn max_dimension(v: Vec3f) -> usize {
    if v.x > v.y {
        if v.x > v.z { 0 } else { 2 }
    } else {
        if v.y > v.z { 1 } else { 2 }
    }
}

pub fn coordinate_system(v1: Vec3f) -> (Vec3f, Vec3f) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vec3f::new(-v1.z, 0.0, v1.x).normalize()
    } else {
        Vec3f::new(0.0, v1.z, -v1.y).normalize()
    };

    let v3 = v1.cross(v2);
    (v2, v3)
}

pub fn faceforward(v1: Vec3f, v2: Vec3f) -> Vec3f {
    if v1.dot(v2) < 0.0 {
        -v1
    } else {
        v1
    }
}

pub fn offset_ray_origin(p: Point3f, p_err: Vec3f, n: Normal3, dir: Vec3f) -> Point3f {
    let d = n.map(|v| v.abs()).dot(p_err);
    let mut offset = d * n.0;
    if dir.dot(n.0) < 0.0 {
        offset = -offset;
    }
    let mut po: Point3f = p + offset;
    for i in 0..3 {
        if offset[i] > 0.0 { po[i] = next_float_up(po[i]) }
        else if offset[i] < 0.0 { po[i] = next_float_down(po[i]) }
    }

    po
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3f,
    pub dir: Vec3f,
    pub t_max: f32,
    pub time: f32,

    // TODO: medium, differentials
}

impl Ray {
    pub fn new(origin: Point3f, dir: Vec3f) -> Self {
        Self {
            origin, dir, t_max: std::f32::INFINITY, time: 0.0
        }
    }
    pub fn at(&self, t: f32) -> Point3f {
        self.origin + (self.dir * t)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Differential {
    pub rx_origin: Point3f,
    pub ry_origin: Point3f,
    pub rx_dir: Vec3f,
    pub ry_dir: Vec3f,
}

/// Ray differentials contain information about two auxiliary rays that represent camera rays
/// offset by one sample in the x and y direction from the main ray on the film plane.
#[derive(Debug)]
pub struct RayDifferential {
    pub ray: Ray,
    pub diff: Option<Differential>,
}

impl RayDifferential {
    pub fn scale_differentials(&mut self, s: Float) {
        if let Some(mut diff) = self.diff {
            diff.rx_origin = self.ray.origin + (diff.rx_origin - self.ray.origin) * s;
            diff.ry_origin = self.ray.origin + (diff.ry_origin - self.ray.origin) * s;
            diff.rx_dir = self.ray.dir + (diff.rx_dir - self.ray.dir) * s;
            diff.ry_dir = self.ray.dir + (diff.ry_dir - self.ray.dir) * s;
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Normal3(pub Vec3f);

impl Normal3 {
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        Self(Vec3f::new(x, y, z))
    }

    pub fn faceforward(self, v: Vec3f) -> Self {
        if self.dot(v) < 0.0 {
            Self(-self.0)
        } else {
            self
        }
    }
}

impl From<[Float; 3]> for Normal3 {
    fn from(v: [f32; 3]) -> Self {
        Self(v.into())
    }
}

impl Deref for Normal3 {
    type Target = Vec3f;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Mul<Float> for Normal3 {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl std::ops::Mul<Normal3> for Float {
    type Output = Normal3;

    fn mul(self, rhs: Normal3) -> Self::Output {
        Normal3(self * rhs.0)
    }
}

impl std::ops::Add<Normal3> for Normal3 {
    type Output = Self;

    fn add(self, rhs: Normal3) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Normal3> for Normal3 {
    type Output = Self;

    fn sub(self, rhs: Normal3) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Normal3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl std::ops::MulAssign<Float> for Normal3 {
    fn mul_assign(&mut self, rhs: Float) {
        self.0 *= rhs;
    }
}

impl From<Vec3f> for Normal3 {
    fn from(v: Vec3f) -> Self {
        Self(v)
    }
}

impl From<Normal3> for Vec3f {
    fn from(n: Normal3) -> Self {
        n.0
    }
}

