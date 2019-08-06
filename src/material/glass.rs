use crate::texture::{Texture, ConstantTexture};
use crate::spectrum::Spectrum;
use std::sync::Arc;
use crate::{Float, SurfaceInteraction};
use crate::material::{Material, TransportMode};
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::{SpecularReflection, SpecularTransmission};
use crate::fresnel::FresnelDielectric;

pub struct GlassMaterial {
    reflectance: Arc<dyn Texture<Spectrum>>,
    transmittance: Arc<dyn Texture<Spectrum>>,
    eta: Arc<dyn Texture<Float>>,
}

impl GlassMaterial {
    pub fn constant(kr: Spectrum, kt: Spectrum, eta: Float) -> Self {
        Self {
            reflectance: Arc::new(ConstantTexture(kr)),
            transmittance: Arc::new(ConstantTexture(kt)),
            eta: Arc::new(ConstantTexture(eta)),
        }
    }
}

impl Material for GlassMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let eta = self.eta.evaluate(si);
        let r = self.reflectance.evaluate(si).clamp_positive();
        let t = self.transmittance.evaluate(si).clamp_positive();
        let mut bsdf = Bsdf::new(si, eta);

        if !r.is_black() {
            let reflection = arena.alloc(SpecularReflection::new(r, FresnelDielectric::new(1.0, eta)));
            bsdf.add(reflection);
        }

        if !t.is_black() {
            let transmission = arena.alloc(SpecularTransmission::new(t, 1.0, eta, mode));
            bsdf.add(transmission);
        }
        bsdf
    }
}