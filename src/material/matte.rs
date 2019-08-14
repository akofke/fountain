use std::sync::Arc;

use bumpalo::Bump;

use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::reflection::bsdf::Bsdf;
use crate::reflection::LambertianReflection;
use crate::spectrum::Spectrum;
use crate::texture::{ConstantTexture, Texture};

pub struct MatteMaterial {
    diffuse: Arc<dyn Texture<Spectrum>>,
    // TODO sigma, bump map
}

impl MatteMaterial {
    pub fn constant(diffuse: Spectrum) -> Self {
        Self {
            diffuse: Arc::new(ConstantTexture(diffuse))
        }
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let mut bsdf = Bsdf::new(si, 1.0);

        let r = self.diffuse.evaluate(si).clamp_positive();
        if !r.is_black() {
            let lambertian = arena.alloc(LambertianReflection { r });
            bsdf.add(lambertian)
        }
        bsdf
    }
}