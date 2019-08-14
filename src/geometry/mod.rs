use crate::{Vec3f, Point3f};
use cgmath::{Matrix4, Transform as cgTransform, InnerSpace, SquareMatrix};
use std::ops::{Deref, Mul};
use crate::Float;

pub mod bounds;
pub mod transform;

pub use bounds::*;
pub use transform::*;
use crate::err_float::{gamma, next_float_up, next_float_down};
use crate::interaction::{SurfaceInteraction, SurfaceHit, DiffGeom, TextureDifferentials};

pub fn distance(p1: Point3f, p2: Point3f) -> Float {
    (p1 - p2).magnitude()
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

#[derive(Copy, Clone)]
pub struct Differential {
    pub rx_origin: Point3f,
    pub ry_origin: Point3f,
    pub rx_dir: Vec3f,
    pub ry_dir: Vec3f,
}

/// Ray differentials contain information about two auxiliary rays that represent camera rays
/// offset by one sample in the x and y direction from the main ray on the film plane.
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

impl std::ops::Neg for Normal3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
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

