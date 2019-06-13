use crate::{Point2f, Vec3f, Point3f, Float};
use crate::geometry::Normal3;

#[derive(Clone, Copy)]
pub struct HitPoint {
    pub p: Point3f,
    pub p_err: Vec3f,
    pub time: Float,
}

pub struct SurfaceInteraction {
    pub hit: HitPoint,

    /// (u, v) coordinates from the parametrization of the surface
    pub uv: Point2f,

    pub wo: Vec3f,

    pub n: Normal3,

    pub geom: DiffGeom,

//    pub shading_n: Normal3,
//
//    pub shading_geom: DiffGeom,


    // shape
    // primitive
    // BSDF
    // BSSRDF
    //


}

impl SurfaceInteraction {
    pub fn new(
        p: Point3f,
        p_err: Vec3f,
        time: Float,
        uv: Point2f,
        wo: Vec3f,
        n: Normal3,
        geom: DiffGeom
    ) -> Self {
        Self {
            hit: HitPoint {p, p_err, time},
            uv,
            wo,
            n,
            geom
        }
    }
}

pub struct DiffGeom {
    pub dpdu: Vec3f,
    pub dpdv: Vec3f,
    pub dndu: Normal3,
    pub dndv: Normal3
}