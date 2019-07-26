use crate::geometry::Normal3;
use crate::material::TransportMode;
use crate::{offset_ray_origin, Float, Point2f, Point3f, Ray, RayDifferential, Vec3f, Vec2f, solve_linear_system_2x2, Differential};
use bumpalo::Bump;
use cgmath::{EuclideanSpace, InnerSpace, Matrix2, Vector2};
use crate::reflection::bsdf::Bsdf;
use crate::primitive::Primitive;

#[derive(Clone, Copy)]
pub struct HitPoint {
    pub p: Point3f,
    pub p_err: Vec3f,
    pub time: Float,
}

impl HitPoint {}

pub struct SurfaceInteraction<'i> {
    pub hit: HitPoint,

    /// (u, v) coordinates from the parametrization of the surface
    pub uv: Point2f,

    pub wo: Vec3f,

    pub n: Normal3,

    pub geom: DiffGeom,

    pub shading_n: Normal3,

    pub shading_geom: DiffGeom,

    pub tex_diffs: Option<TextureDifferentials>,

    // TODO: CHANGE THIS
    pub primitive: Option<&'i dyn Primitive>
    // shape
    // primitive
    // BSDF
    // BSSRDF
    //
}

impl<'i> SurfaceInteraction<'i> {
    pub fn new(
        p: Point3f,
        p_err: Vec3f,
        time: Float,
        uv: Point2f,
        wo: Vec3f,
        n: Normal3,
        geom: DiffGeom,
    ) -> Self {
        Self {
            hit: HitPoint { p, p_err, time },
            uv,
            wo,
            n,
            geom,

            shading_n: n,
            shading_geom: geom,

            tex_diffs: None,
            primitive: None
        }
    }

    pub fn spawn_ray(&self, dir: Vec3f) -> Ray {
        let o = offset_ray_origin(&self.hit.p, &self.hit.p_err, &self.n, &dir);
        Ray {
            origin: o,
            dir,
            t_max: std::f32::INFINITY,
            time: self.hit.time,
        }
    }

    pub fn spawn_ray_with_dfferentials(&self, dir: Vec3f, diff: Option<Differential>) -> RayDifferential {
        let ray = self.spawn_ray(dir);
        RayDifferential { ray, diff }
    }

    pub fn compute_scattering_functions<'a>(
        &mut self,
        ray: &RayDifferential,
        arena: &'a Bump,
        allow_multiple_lobes: bool,
        mode: TransportMode,
    ) -> Option<Bsdf<'a>> {
        self.tex_diffs = self.compute_tex_differentials(ray);
        let material = self.primitive.expect("Should have a prim at this point").material()?;
        Some(material.compute_scattering_functions(self, arena, mode, allow_multiple_lobes))
    }

    fn compute_tex_differentials(&self, ray: &RayDifferential) -> Option<TextureDifferentials> {
        let n = self.n;
        let diff = ray.diff?;
        let d = self.n.dot(self.hit.p.to_vec());

        let px = {
            let tx = -(self.n.dot(diff.rx_origin.to_vec()) - d) / self.n.dot(diff.rx_dir);
            diff.rx_origin + tx * diff.rx_dir
        };

        let py = {
            let ty = -(self.n.dot(diff.ry_origin.to_vec()) - d) / self.n.dot(diff.ry_dir);
            diff.ry_origin + ty * diff.ry_dir
        };

        let dpdx = px - self.hit.p;
        let dpdy = py - self.hit.p;

        let dim = if n.x.abs() > n.y.abs() && n.x.abs() > n.z.abs() {
            (1, 2)
        } else if n.y.abs() > n.z.abs() {
            (0, 2)
        } else {
            (0, 1)
        };

        let dpdu = self.geom.dpdu;
        let dpdv = self.geom.dpdv;
        let A = Matrix2::from_cols(
            Vector2::new(dpdu[dim.0], dpdu[dim.1]),
            Vector2::new(dpdv[dim.0], dpdv[dim.1])
        );

        let bx = Vec2f::new(dpdx[dim.0], dpdx[dim.1]);
        let by = Vec2f::new(dpdy[dim.0], dpdy[dim.1]);

        // TODO: can we ever have p differentials without uv differentials?
        let (dudx, dvdx) = solve_linear_system_2x2(A, bx)?.into();
        let (dudy, dvdy) = solve_linear_system_2x2(A, by)?.into();
        Some(TextureDifferentials {
            dpdx,
            dpdy,

            dudx,
            dvdx,

            dudy,
            dvdy
        })
    }
}

#[derive(Clone, Copy)]
pub struct DiffGeom {
    pub dpdu: Vec3f,
    pub dpdv: Vec3f,
    pub dndu: Normal3,
    pub dndv: Normal3,
}

/// Partial derivatives used for texture antialiasing.
#[derive(Clone, Copy)]
pub struct TextureDifferentials {
    pub dpdx: Vec3f,
    pub dpdy: Vec3f,

    pub dudx: Float,
    pub dvdx: Float,

    pub dudy: Float,
    pub dvdy: Float,
}
