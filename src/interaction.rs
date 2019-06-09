use crate::{Point2f, Vec3, Point3f};
use crate::geometry::Normal3;

pub struct HitPoint {
    pub p: Point3f,
    pub time: f32,
    pub p_err: Vec3,
}

pub struct SurfaceInteraction {

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
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal3,
    pub dndv: Normal3
}