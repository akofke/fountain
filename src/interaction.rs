use crate::{Point2f, Vec3f, Point3f};
use crate::geometry::Normal3;

#[derive(Clone, Copy)]
pub struct HitPoint {
    pub p: Point3f,
    pub time: f32,
    pub p_err: Vec3f,
}

pub struct SurfaceInteraction {
    pub hit: HitPoint,

    /// (u, v) coordinates from the parametrization of the surface
    pub uv: Point2f,

    pub n: Normal3,

    pub geom: DiffGeom,

    pub shading_n: Normal3,

    pub shading_geom: DiffGeom,


    // shape
    // primitive
    // BSDF
    // BSSRDF
    //


}

pub struct DiffGeom {
    pub dpdu: Vec3f,
    pub dpdv: Vec3f,
    pub dndu: Normal3,
    pub dndv: Normal3
}