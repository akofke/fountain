use crate::{Vec3f, Point3f};
use nalgebra::Transform3;
use nalgebra::Point3;
use nalgebra::Matrix4;
use std::ops::Deref;
use crate::Float;

pub mod bounds;

pub use bounds::*;
use crate::err_float::{gamma, next_float_up, next_float_down};
use crate::interaction::{SurfaceInteraction, HitPoint, DiffGeom};

pub fn distance(p1: Point3f, p2: Point3f) -> Float {
    (p1 - p2).norm()
}

pub fn offset_ray_origin(p: &Point3f, p_err: &Vec3f, n: &Normal3, dir: &Vec3f) -> Point3f {
    let d = n.abs().dot(p_err);
    let mut offset = d * n.0;
    if dir.dot(n) < 0.0 {
        offset = -offset;
    }
    let mut po: Point3f = p + offset;
    for i in 0..3 {
        if offset[i] > 0.0 { po[i] = next_float_up(po[i]) }
        else if offset[i] < 0.0 { po[i] = next_float_down(po[i]) }
    }

    po
}

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


pub struct Normal3(pub Vec3f);

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

impl Transformable for Normal3 {
    fn transform(&self, t: Transform) -> Self {
        t.transform_normal(self)
    }
}

impl Transformable<(Self, Vec3f)> for Point3f {
    /// Transform a Point, giving the transformed point and a vector of the absolute error
    /// introduced by the transformation
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

impl Transformable<(Point3f, Vec3f)> for (Point3f, Vec3f) {
    /// Transform a point given its existing absolute error, producing the transformed point
    /// and its new absolute error
    fn transform(&self, t: Transform) -> (Point3f, Vec3f) {
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

impl Transformable<(Vec3f, Vec3f)> for Vec3f {
    fn transform(&self, t: Transform) -> (Vec3f, Vec3f) {
        let vt = t.t.transform_vector(self);
        let m = t.t;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_abs_sum = (m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs();
        let y_abs_sum = (m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs();
        let z_abs_sum = (m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs();

        let v_error = vec3f!(x_abs_sum, y_abs_sum, z_abs_sum) * gamma(3);
        (vt, v_error)
    }
}

impl Transformable<(Vec3f, Vec3f)> for (Vec3f, Vec3f) {
    /// Transform a vector given its existing absolute error, producing the transformed vector
    /// and its new absolute error
    fn transform(&self, t: Transform) -> (Vec3f, Vec3f) {
        let (v, verr) = self;
        let vt = t.t.transform_vector(v);
        let m = t.t;

        let xerr = (gamma(3) + 1.0) *
            (m[(0, 0)] * verr.x).abs() + (m[(0, 1)] * verr.y).abs() + (m[(0, 2)] * verr.z).abs() +
            gamma(3) * (m[(0, 0)] * v.x).abs() + (m[(0, 1)] * v.y).abs() + (m[(0, 2)] * v.z).abs();

        let yerr = (gamma(3) + 1.0) *
            (m[(1, 0)] * verr.x).abs() + (m[(1, 1)] * verr.y).abs() + (m[(1, 2)] * verr.z).abs() +
            gamma(3) * (m[(1, 0)] * v.x).abs() + (m[(1, 1)] * v.y).abs() + (m[(1, 2)] * v.z).abs();

        let zerr = (gamma(3) + 1.0) *
            (m[(2, 0)] * verr.x).abs() + (m[(2, 1)] * verr.y).abs() + (m[(2, 2)] * verr.z).abs() +
            gamma(3) * (m[(2, 0)] * v.x).abs() + (m[(2, 1)] * v.y).abs() + (m[(2, 2)] * v.z).abs();

        let v_error = vec3f!(xerr, yerr, zerr);
        (vt, v_error)
    }
}

impl Transformable<(Ray, Vec3f, Vec3f)> for &Ray {
    fn transform(&self, t: Transform) -> (Ray, Vec3f, Vec3f) {
        let (mut ot, o_err) = self.origin.transform(t);
        let (dir_t, dir_err) = self.dir.transform(t);
        let mut tmax = self.t_max;

        let len_sq = dir_t.norm_squared();
        if len_sq > 0.0 {
            let dt = dir_t.abs().dot(&o_err) / len_sq;
            ot += dir_t * dt;
            tmax -= dt; // why was this commented out in pbrt source code but not book?
        }
        let ray_t = Ray { origin: ot, dir: dir_t, t_max: tmax, time: self.time };
        (ray_t, o_err, dir_err)
    }
}

impl Transformable for HitPoint {
    fn transform(&self, t: Transform) -> Self {
        let (pt, pterr) = (self.p, self.p_err).transform(t);
        HitPoint { p: pt, p_err: pterr, time: self.time }
    }
}

impl Transformable for DiffGeom {
    fn transform(&self, t: Transform) -> Self {
        Self {
            dpdu: self.dpdu.transform(t),
            dpdv: self.dpdv.transform(t),
            dndu: self.dndu.transform(t),
            dndv: self.dndv.transform(t)
        }
    }
}

impl Transformable for SurfaceInteraction {
    fn transform(&self, t: Transform) -> Self {
        Self {
            hit: self.hit.transform(t),
            uv: self.uv,
            wo: Transformable::<Vec3f>::transform(&self.wo, t).normalize(),
            n: self.n.transform(t),
            geom: self.geom.transform(t)
        }
    }
}
