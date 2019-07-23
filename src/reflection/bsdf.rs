use crate::{Float, Normal3, Vec3f, Point2f};
use arrayvec::ArrayVec;
use crate::reflection::{BxDF, BxDFType, ScatterSample};
use crate::interaction::SurfaceInteraction;
use cgmath::InnerSpace;
use crate::spectrum::Spectrum;

pub struct Bsdf<'a> {

    /// Index of refraction over the boundary
    pub eta: Float,

    /// Shading normal
    ns: Normal3,

    /// Geometry normal
    ng: Normal3,

    /// s orthonormal basis vector with the shading normal
    ss: Vec3f,

    /// t orthonormal basis vector with the shading normal
    ts: Vec3f,

    bxdfs: ArrayVec<[&'a dyn BxDF; 8]>
}

impl<'a> Bsdf<'a> {

    pub fn new(si: &SurfaceInteraction, eta: Float) -> Self {
        let ns = si.shading_n;
        let ng = si.n;
        let ss = si.shading_geom.dpdu.normalize();
        let ts = ns.cross(ss).normalize();
        let bxdfs = ArrayVec::new();

        Self {
            eta,
            ns,
            ng,
            ss,
            ts,
            bxdfs
        }
    }

    pub fn add(&mut self, bxdf: &'a dyn BxDF) {
        self.bxdfs.push(bxdf);
    }

    pub fn num_components(&self, flags: BxDFType) -> usize {
        self.bxdfs.as_slice().iter().filter(|bxdf| bxdf.matches_flags(flags)).count()
    }

    pub fn world_to_local(&self, v: Vec3f) -> Vec3f {
        Vec3f::new(v.dot(self.ss), v.dot(self.ts), v.dot(self.ns.0))
    }

    pub fn local_to_world(&self, v: Vec3f) -> Vec3f {
        let x = self.ss.x * v.x + self.ts.x * v.y + self.ns.x * v.z;
        let y = self.ss.y * v.x + self.ts.y * v.y + self.ns.y * v.z;
        let z = self.ss.z * v.x + self.ts.z * v.y + self.ns.z * v.z;
        Vec3f::new(x, y, z)
    }

    pub fn f(&self, wo_world: Vec3f, wi_world: Vec3f, flags: BxDFType) -> Spectrum {
        let wi = self.world_to_local(wi_world);
        let wo = self.world_to_local(wo_world);
        if wo.z == 0.0 { return Spectrum::new(0.0) }

        let reflect = wi_world.dot(self.ng.into()) * wo_world.dot(self.ng.into()) > 0.0;

        self.bxdfs.as_slice().iter()
            .filter(|bxdf| bxdf.matches_flags(flags))
            .filter(|bxdf| {
                (reflect && bxdf.get_type().contains(BxDFType::REFLECTION))
                || (!reflect && bxdf.get_type().contains(BxDFType::TRANSMISSION))
            })
            .map(|bxdf| bxdf.f(wo, wi))
            .sum()
    }

    pub fn sample_f(&self, wo_world: Vec3f, u: Point2f, flags: BxDFType) -> Option<ScatterSample> {
        let matching_comps = self.num_components(flags) as Float;
        if matching_comps == 0.0 { return None }

        let comp = (u[0] * (matching_comps)).floor().min(matching_comps - 1.0) as usize;

        let bxdf: &dyn BxDF = *self.bxdfs.as_slice().iter()
            .filter(|bxdf| bxdf.matches_flags(flags))
            .nth(comp).unwrap();

        let u_remapped = Point2f::new(u[0] * matching_comps - (comp as Float), u[1]);

        let wo = self.world_to_local(wo_world);
        let f = bxdf.sample_f()

    }
}
