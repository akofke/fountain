use std::sync::Arc;

use bumpalo::Bump;

use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::reflection::bsdf::Bsdf;
use crate::reflection::{LambertianReflection, OrenNayar};
use crate::spectrum::Spectrum;
use crate::Float;
use crate::texture::{ConstantTexture, Texture, TextureRef};
use cgmath::Deg;

pub struct MatteMaterial {
    diffuse: Arc<dyn Texture<Output = Spectrum>>,
    sigma: TextureRef<Float>
    // TODO sigma, bump map
}

impl MatteMaterial {
    pub fn new(
        diffuse: Arc<dyn Texture<Output=Spectrum>>,
        sigma: TextureRef<Float>,
    ) -> Self {
        Self { diffuse, sigma }
    }
    pub fn constant(diffuse: Spectrum) -> Self {
        Self::new(
            Arc::new(ConstantTexture(diffuse)),
            Arc::new(ConstantTexture(0.0))
        )
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let mut bsdf = Bsdf::new(si, 1.0);

        let r = self.diffuse.evaluate(si).clamp_positive();
        let sigma = self.sigma.evaluate(si).clamp(0.0, 90.0);
        if !r.is_black() {
            if sigma == 0.0 {
                let lambertian = arena.alloc(LambertianReflection { r });
                bsdf.add(lambertian);
            } else {
                bsdf.add(arena.alloc(
                    OrenNayar::new(r, Deg(sigma))
                ));
            }
        }
        bsdf
    }
}