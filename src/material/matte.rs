use std::rc::Rc;
use crate::texture::Texture;
use crate::spectrum::Spectrum;
use crate::material::{Material, TransportMode};
use crate::interaction::SurfaceInteraction;
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::LambertianReflection;

pub struct MatteMaterial {
    diffuse: Rc<Texture<Spectrum>>,
    // TODO sigma, bump map
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(&self, si: &SurfaceInteraction, arena: &Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf {
        let mut bsdf = Bsdf::new(si, 0.0);

        let r = self.diffuse.evaluate(si).clamp_positive();
        if !r.is_black() {
            let lambertian = arena.alloc(LambertianReflection { r });
            bsdf.add(lambertian)
        }
        unimplemented!()
    }
}