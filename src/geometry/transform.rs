use crate::{Float, Point3f, Vec3f, Normal3, Bounds3f, Ray, SurfaceInteraction, ComponentWiseExt, RayDifferential, Differential};
use cgmath::{Matrix4, SquareMatrix, InnerSpace, Transform as cgTransform, Zero, Rad};
use crate::err_float::gamma;
use crate::interaction::{SurfaceHit, DiffGeom, TextureDifferentials};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub t: Matrix4<Float>,
    pub invt: Matrix4<Float>
}

const IDENTITY_MAT4: Matrix4<Float> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
);

impl Transform {

    pub const IDENTITY: Self = Transform::new(IDENTITY_MAT4, IDENTITY_MAT4);

    pub fn from_mat(mat: Matrix4<Float>) -> Self {
        let m_inv = mat.invert().expect("Could not invert matrix");
        Self::new(mat, m_inv)
    }

    pub const fn new(mat: Matrix4<Float>, mat_inv: Matrix4<Float>) -> Self {
        let t = mat;
        let invt = mat_inv;
        Self { t, invt }
    }

    pub fn look_at(pos: Point3f, look_at: Point3f, up: Vec3f) -> Self {
        let col3 = pos.to_homogeneous();
        let dir = (look_at - pos).normalize();
        let right = up.normalize().cross(dir).normalize();
        let new_up = dir.cross(right);

        let col0 = right.extend(0.0);
        let col1 = new_up.extend(0.0);
        let col2 = dir.extend(0.0);

        let mat = Matrix4::from_cols(col0, col1, col2, col3);
        let minv = mat.inverse_transform().unwrap();
        Self::new(minv, mat)
    }

    pub fn camera_look_at(pos: Point3f, look_at: Point3f, up: Vec3f) -> Self {
        Self::look_at(pos, look_at, up).inverse()
    }

    pub fn translate(delta: Vec3f) -> Self {
        let m = Matrix4::from_translation(delta);
        let m_inv = Matrix4::from_translation(-delta);
        Self::new(m, m_inv)
    }

    pub fn scale(sx: Float, sy: Float, sz: Float) -> Self {
        let m = Matrix4::from_nonuniform_scale(sx, sy, sz);
        let m_inv = Matrix4::from_nonuniform_scale(1.0 / sx, 1.0 / sy, 1.0 / sz);
        Self::new(m, m_inv)
    }

    pub fn rotate_x(theta: impl Into<Rad<Float>>) -> Self {
        let m = Matrix4::from_angle_x(theta);
        let m_inv = m.inverse_transform().unwrap();
        Self::new(m, m_inv)
    }

    pub fn rotate_y(theta: impl Into<Rad<Float>>) -> Self {
        let m = Matrix4::from_angle_y(theta);
        let m_inv = m.inverse_transform().unwrap();
        Self::new(m, m_inv)
    }

    pub fn rotate_z(theta: impl Into<Rad<Float>>) -> Self {
        let m = Matrix4::from_angle_z(theta);
        let m_inv = m.inverse_transform().unwrap();
        Self::new(m, m_inv)
    }

    pub fn fit_to_bounds(subject: Bounds3f, target: Bounds3f) -> Self {
        let displacement = target.centroid() - subject.centroid();
        let scale = target.diagonal().magnitude() / subject.diagonal().magnitude();
        Self::translate(displacement).then(Self::scale(scale, scale, scale))
    }

    pub fn perspective(fov: Float, near: Float, far: Float) -> Self {
        let mat = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, far / (far-near), 1.0,
            0.0, 0.0, -far * near / (far - near), 0.0
        );

        let inv_tan_ang = 1.0 / (fov.to_radians() / 2.0).tan();
        Transform::scale(inv_tan_ang, inv_tan_ang, 1.0) * Self::from_mat(mat)
    }

    pub fn identity() -> Self {
        Self::new(Matrix4::identity(), Matrix4::identity())
    }

    pub fn inverse(&self) -> Self {
        Self::new(self.invt, self.t)
    }

    pub fn swaps_handedness(&self) -> bool {
        self.t.determinant() < 0.0
    }

    pub fn then(self, next: Self) -> Self {
        next * self
    }

    pub fn transform_normal(&self, n: &Normal3) -> Normal3 {
        // transform by the transpose of the inverse
        let x = self.invt[0][0]*n.x + self.invt[1][0]*n.y + self.invt[2][0]*n.z;
        let y = self.invt[0][1]*n.x + self.invt[1][1]*n.y + self.invt[2][1]*n.z;
        let z = self.invt[0][2]*n.x + self.invt[1][2]*n.y + self.invt[2][2]*n.z;
        Normal3(vec3f!(x, y, z))
    }

    pub fn transform<T: Transformable>(&self, obj: T) -> T {
        obj.transform(*self)
    }

    pub fn tf_exact_to_err<T: TransformableErr>(&self, obj: T) -> (T, T::Err) {
        obj.tf_exact_to_err(*self)
    }

    pub fn tf_err_to_err<T: TransformableErr>(&self, obj: T, err: T::Err) -> (T, T::Err) {
        obj.tf_err_to_err(err, *self)
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new(self.t * rhs.t, rhs.invt * self.invt)
    }
}

// TODO: decide on what should be references vs by value
pub trait Transformable: Sized {
    fn transform(&self, t: Transform) -> Self;
}

pub trait TransformableErr: Transformable {
    type Err;

    fn tf_exact_to_err(&self, t: Transform) -> (Self, Self::Err);

    fn tf_err_to_err(&self, err: Self::Err, t: Transform) -> (Self, Self::Err);
}

impl Transformable for Vec3f {
    fn transform(&self, t: Transform) -> Self {
        t.t.transform_vector(*self)
    }
}

impl TransformableErr for Vec3f {
    type Err = Vec3f;

    fn tf_exact_to_err(&self, tf: Transform) -> (Self, Self::Err) {
        let vt = tf.t.transform_vector(*self);
        let m = tf.t;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_abs_sum = (m[0][0] * x).abs() + (m[1][0] * y).abs() + (m[2][0] * z).abs();
        let y_abs_sum = (m[0][1] * x).abs() + (m[1][1] * y).abs() + (m[2][1] * z).abs();
        let z_abs_sum = (m[0][2] * x).abs() + (m[1][2] * y).abs() + (m[2][2] * z).abs();

        let v_error = vec3f!(x_abs_sum, y_abs_sum, z_abs_sum) * gamma(3);
        (vt, v_error)
    }

    // TODO: can write partially in terms of tf_exact_to_err
    fn tf_err_to_err(&self, err: Self::Err, tf: Transform) -> (Self, Self::Err) {
        let v = self;
        let verr = err;
        let vt = tf.t.transform_vector(*v);
        let m = tf.t;

        let xerr = (gamma(3) + 1.0) *
            ((m[0][0] * verr.x).abs() + (m[1][0] * verr.y).abs() + (m[2][0] * verr.z).abs()) +
            gamma(3) * ((m[0][0] * v.x).abs() + (m[1][0] * v.y).abs() + (m[2][0] * v.z).abs());

        let yerr = (gamma(3) + 1.0) *
            ((m[0][1] * verr.x).abs() + (m[1][1] * verr.y).abs() + (m[2][1] * verr.z).abs()) +
            gamma(3) * ((m[0][1] * v.x).abs() + (m[1][1] * v.y).abs() + (m[2][1] * v.z).abs());

        let zerr = (gamma(3) + 1.0) *
            ((m[0][2] * verr.x).abs() + (m[1][2] * verr.y).abs() + (m[2][2] * verr.z).abs()) +
            gamma(3) * ((m[0][2] * v.x).abs() + (m[1][2] * v.y).abs() + (m[2][2] * v.z).abs());

        let v_error = vec3f!(xerr, yerr, zerr);
        (vt, v_error)
    }
}

impl Transformable for Point3f {
    fn transform(&self, t: Transform) -> Self { t.t.transform_point(*self) }
}

impl TransformableErr for Point3f {
    type Err = Vec3f;

    fn tf_exact_to_err(&self, tf: Transform) -> (Self, Self::Err) {
        let pt = tf.t.transform_point(*self);
        let m = tf.t;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let x_abs_sum = (m[0][0] * x).abs() + (m[1][0] * y).abs() + (m[2][0] * z).abs() + m[3][0].abs();
        let y_abs_sum = (m[0][1] * x).abs() + (m[1][1] * y).abs() + (m[2][1] * z).abs() + m[3][1].abs();
        let z_abs_sum = (m[0][2] * x).abs() + (m[1][2] * y).abs() + (m[2][2] * z).abs() + m[3][2].abs();

        let p_error = vec3f!(x_abs_sum, y_abs_sum, z_abs_sum) * gamma(3);
        (pt, p_error)
    }

    fn tf_err_to_err(&self, err: Self::Err, tf: Transform) -> (Self, Self::Err) {
        let p = self;
        let perr = err;
        let pt = tf.t.transform_point(*p);
        let m = tf.t;

        let xerr = (gamma(3) + 1.0) *
            ((m[0][0]).abs() * perr.x + (m[1][0]).abs() * perr.y + (m[2][0]).abs() * perr.z) +
            gamma(3) * ((m[0][0] * p.x).abs() + (m[1][0] * p.y).abs() + (m[2][0] * p.z).abs() + m[3][0].abs());

        let yerr = (gamma(3) + 1.0) *
            ((m[0][1]).abs() * perr.x + (m[1][1]).abs() * perr.y + (m[2][1]).abs() * perr.z) +
            gamma(3) * ((m[0][1] * p.x).abs() + (m[1][1] * p.y).abs() + (m[2][1] * p.z).abs() + m[3][1].abs());

        let zerr = (gamma(3) + 1.0) *
            ((m[0][2] * perr.x).abs() + (m[1][2] * perr.y).abs() + (m[2][2] * perr.z).abs()) +
            gamma(3) * ((m[0][2] * p.x).abs() + (m[1][2] * p.y).abs() + (m[2][2] * p.z).abs() + m[3][2].abs());

        let p_error = vec3f!(xerr, yerr, zerr);
        (pt, p_error)
    }
}

impl Transformable for Normal3 {
    fn transform(&self, t: Transform) -> Self {
        t.transform_normal(self)
    }
}

impl Transformable for Bounds3f {
    fn transform(&self, t: Transform) -> Self {
        self.iter_corners().fold(Bounds3f::empty(), |b, p| {
            let pt = t.transform(p);
            b.join_point(pt)
        })
    }
}

impl TransformableErr for Ray {
    type Err = (Vec3f, Vec3f);

    fn tf_exact_to_err(&self, t: Transform) -> (Self, Self::Err) {
        let (mut ot, o_err) = t.tf_exact_to_err(self.origin);
        let (dir_t, dir_err) = t.tf_exact_to_err(self.dir);
        let mut tmax = self.t_max;

        let len_sq = dir_t.magnitude2();
        if len_sq > 0.0 {
            let dt = dir_t.abs().dot(o_err) / len_sq;
            ot += dir_t * dt;
            tmax -= dt; // why was this commented out in pbrt source code but not book?
        }
        let ray_t = Ray { origin: ot, dir: dir_t, t_max: tmax, time: self.time };
        (ray_t, (o_err, dir_err))
    }

    fn tf_err_to_err(&self, err: Self::Err, t: Transform) -> (Self, Self::Err) {
        unimplemented!()
    }
}

impl Transformable for Ray {
    fn transform(&self, t: Transform) -> Ray {
        let (mut ot, o_err) = self.origin.tf_exact_to_err(t);
        let dir: Vec3f = self.dir.transform(t);
        let mut t_max = self.t_max;

        // Offset ray origin to edge of error bounds and compute t_max
        let len_sq = dir.magnitude2();
        if len_sq > 0.0 {
            let dt = dir.map(|v| v.abs()).dot(o_err) / len_sq;
            ot += dir * dt;
            t_max -= dt;
        }

        Ray { origin: ot, dir, t_max, time: self.time }
    }
}

impl Transformable for RayDifferential {
    fn transform(&self, t: Transform) -> Self {
        RayDifferential {
            ray: self.ray.transform(t),
            diff: self.diff.map(|diff| {
                Differential {
                    rx_origin: diff.rx_origin.transform(t),
                    ry_origin: diff.ry_origin.transform(t),
                    rx_dir: diff.rx_dir.transform(t),
                    ry_dir: diff.ry_dir.transform(t),
                }
            })
        }
    }
}

impl Transformable for SurfaceHit {
    fn transform(&self, t: Transform) -> Self {
        let (pt, pterr) = t.tf_err_to_err(self.p, self.p_err);
        let n = t.transform(self.n).normalize().into();
        SurfaceHit { p: pt, p_err: pterr, time: self.time, n }
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

impl Transformable for TextureDifferentials {
    fn transform(&self, t: Transform) -> Self {
        Self {
            dpdx: self.dpdx.transform(t),
            dpdy: self.dpdy.transform(t),
            ..*self
        }
    }
}

impl Transformable for SurfaceInteraction<'_> {
    fn transform(&self, t: Transform) -> Self {
        Self {
            hit: self.hit.transform(t),
            uv: self.uv,
            wo: t.transform(self.wo).normalize(),
            geom: self.geom.transform(t),

            shading_n: self.shading_n.transform(t).normalize().into(),
            shading_geom: self.shading_geom.transform(t),

            tex_diffs: self.tex_diffs.transform(t),
            primitive: self.primitive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::vec3;
    use cgmath::{assert_abs_diff_eq, assert_ulps_eq};

    #[test]
    fn test_look_at() {
        let pos = (0.0, 0.0, -1.0).into();
        let tf = Transform::camera_look_at(
            pos,
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
        );
//        dbg!(tf);
        let dir = Vec3f::new(0.0, 0.0, 1.0); // positive z-axis
        let expected = Vec3f::new(0.0, 0.0, 1.0);

        let ray = Ray::new(Point3f::new(0.0, 0.0, 0.0), dir);

        let world_ray = ray.transform(tf);

        assert_abs_diff_eq!(world_ray.dir, expected, epsilon = 0.00001);
        assert_abs_diff_eq!(world_ray.origin, pos, epsilon = 0.00001);
    }

    #[test]
    fn test_point_transform() {
        // translate, then scale
        let tf = Transform::scale(2.0, 2.0, 2.0) *
            Transform::translate(vec3(1.0, 1.0, 1.0));


        let p = Point3f::new(1.0, 1.0, 1.0);
        let perr = Vec3f::new(0.0001, 0.0001, 0.0001);
        let (pt, pterr) = tf.tf_err_to_err(p, perr);

        assert_abs_diff_eq!(Point3f::new(4.0, 4.0, 4.0), pt, epsilon = 0.00001);
        assert_abs_diff_eq!(2.0 * perr, pterr, epsilon = 0.000001);
    }

    #[test]
    fn test_vec_transform() {
        // translate, then scale. Translate should do nothing as opposed to point.
        let tf = Transform::scale(2.0, 2.0, 2.0) *
            Transform::translate(vec3(1.0, 1.0, 1.0));


        let v = Vec3f::new(1.0, 1.0, 1.0);
        let verr = Vec3f::new(0.0001, 0.0001, 0.0001);
        let (vt, vterr) = tf.tf_err_to_err(v, verr);

        assert_abs_diff_eq!(Vec3f::new(2.0, 2.0, 2.0), vt, epsilon = 0.00001);
        assert_abs_diff_eq!(2.0 * verr, vterr, epsilon = 0.000001);
    }

    #[test]
    fn test_identity() {
        let tf = Transform::IDENTITY;
        let p = Point3f::new(0.0, 0.0, 0.0);

        let pt = tf.transform(p);
        assert_abs_diff_eq!(Point3f::new(0.0, 0.0, 0.0), pt, epsilon = 0.000001);
    }
}
