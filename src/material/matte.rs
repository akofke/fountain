use std::rc::Rc;
use crate::texture::{Texture, ConstantTexture};
use crate::spectrum::Spectrum;
use crate::material::{Material, TransportMode};
use crate::interaction::SurfaceInteraction;
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::LambertianReflection;
use std::sync::Arc;

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
        let mut bsdf = Bsdf::new(si, 0.0);

        let r = self.diffuse.evaluate(si).clamp_positive();
        if !r.is_black() {
            let lambertian = arena.alloc(LambertianReflection { r });
            bsdf.add(lambertian)
        }
        bsdf
    }
}