use crate::texture::Texture;
use crate::spectrum::Spectrum;
use std::sync::Arc;
use crate::material::{Material, TransportMode};
use crate::SurfaceInteraction;
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::SpecularReflection;
use crate::fresnel::FresnelNoOp;

pub struct MirrorMaterial {
    reflectance: Arc<dyn Texture<Spectrum>>,
}

impl MirrorMaterial {
    pub fn new(reflectance: Arc<dyn Texture<Spectrum>>) -> Self {
        Self { reflectance }
    }
}

impl Material for MirrorMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let mut bsdf = Bsdf::new(si, 1.0);
        let r = self.reflectance.evaluate(si).clamp_positive();
        if !r.is_black() {
            let reflection = arena.alloc(SpecularReflection::new(r, FresnelNoOp));
            bsdf.add(reflection);
        }
        bsdf
    }
}