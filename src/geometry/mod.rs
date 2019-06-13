use crate::{Vec3f, Point3f};
use nalgebra::Transform3;
use nalgebra::Point3;
use nalgebra::Matrix4;
use std::ops::Deref;
use crate::Float;

pub mod bounds;

pub use bounds::*;
use crate::err_float::gamma;

pub struct Ray {
    pub origin: Point3<f32>,
    pub dir: Vec3f,
    pub t_max: f32,
    pub time: f32,

    // TODO: medium, differentials
}

impl Ray {
    pub fn new(origin: Point3<f32>, dir: Vec3f) -> Self {
        Self {
            origin, dir, t_max: std::f32::INFINITY, time: 0.0
        }
    }
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + (self.dir * t)
    }
}


pub struct Normal3(Vec3f);

impl Normal3 {
}

impl Deref for Normal3 {
    type Target = Vec3f;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub t: Transform3<f32>,
    pub invt: Transform3<f32>
}

impl Transform {

    pub fn translate(delta: Vec3f) -> Self {
        let m = Matrix4::new_translation(&delta);
        let m_inv = Matrix4::new_translation(&-delta);
        let t = Transform3::from_matrix_unchecked(m);
        let invt = Transform3::from_matrix_unchecked(m_inv);
        Self { t, invt }
    }

    pub fn transform_normal(&self, n: &Normal3) -> Normal3 {
        // transform by the transpose of the inverse
        let x = self.invt[(0, 0)]*n.x + self.invt[(1, 0)]*n.y + self.invt[(2, 0)]*n.z;
        let y = self.invt[(0, 1)]*n.x + self.invt[(1, 1)]*n.y + self.invt[(2, 1)]*n.z;
        let z = self.invt[(0, 2)]*n.x + self.invt[(1, 2)]*n.y + self.invt[(2, 2)]*n.z;
        Normal3(vec3f!(x, y, z))
    }
}

pub trait Transformable<O=Self> {
    fn transform(&self, t: Transform) -> O;
}

impl Transformable for Vec3f {
    fn transform(&self, t: Transform) -> Self {
        t.t.transform_vector(&self)
    }
}

impl Transformable for Point3f {
    fn transform(&self, t: Transform) -> Self { t.t.transform_point(&self) }
}

impl Transformable<(Self, Vec3f)> for Point3f {
    fn transform(&self, t: Transform) -> (Point3f, Vec3f) {
        let pt = t.t.transform_point(&self);
        let m = t.t;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_abs_sum = (m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs() + m[(0, 3)].abs();
        let y_abs_sum = (m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs() + m[(1, 3)].abs();
        let z_abs_sum = (m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs() + m[(2, 3)].abs();

        let p_error = vec3f!(x_abs_sum, y_abs_sum, z_abs_sum) * gamma(3);
        (pt, p_error)
    }
}

impl Transformable<(Self, Vec3f)> for (Point3f, Vec3f) {
    fn transform(&self, t: Transform) -> Self {
        let (p, perr) = self;
        let pt = t.t.transform_point(p);
        let m = t.t;

        let xerr = (gamma(3) + 1.0) *
            (m[(0, 0)] * perr.x).abs() + (m[(0, 1)] * perr.y).abs() + (m[(0, 2)] * perr.z).abs() +
            gamma(3) * (m[(0, 0)] * p.x).abs() + (m[(0, 1)] * p.y).abs() + (m[(0, 2)] * p.z).abs() + m[(0, 3)].abs();

        let yerr = (gamma(3) + 1.0) *
            (m[(1, 0)] * perr.x).abs() + (m[(1, 1)] * perr.y).abs() + (m[(1, 2)] * perr.z).abs() +
            gamma(3) * (m[(1, 0)] * p.x).abs() + (m[(1, 1)] * p.y).abs() + (m[(1, 2)] * p.z).abs() + m[(1, 3)].abs();

        let zerr = (gamma(3) + 1.0) *
            (m[(2, 0)] * perr.x).abs() + (m[(2, 1)] * perr.y).abs() + (m[(2, 2)] * perr.z).abs() +
            gamma(3) * (m[(2, 0)] * p.x).abs() + (m[(2, 1)] * p.y).abs() + (m[(2, 2)] * p.z).abs() + m[(2, 3)].abs();

        let p_error = vec3f!(xerr, yerr, zerr);
        (pt, p_error)
    }
}


