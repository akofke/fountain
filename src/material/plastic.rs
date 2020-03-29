use crate::texture::TextureRef;
use crate::spectrum::Spectrum;
use crate::{Float, SurfaceInteraction};
use crate::material::{Material, TransportMode};
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::{LambertianReflection, MicrofacetReflection};
use crate::fresnel::FresnelDielectric;
use crate::reflection::microfacet::TrowbridgeReitzDistribution;

pub struct PlasticMaterial {
    kd: TextureRef<Spectrum>,
    ks: TextureRef<Spectrum>,
    roughness: TextureRef<Float>,
    remap_roughness: bool,
}

impl PlasticMaterial {
    pub fn new(kd: TextureRef<Spectrum>, ks: TextureRef<Spectrum>, roughness: TextureRef<Float>, remap_roughness: bool) -> Self {
        PlasticMaterial { kd, ks, roughness, remap_roughness }
    }
}

impl Material for PlasticMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let mut bsdf = Bsdf::new(si, 1.0);
        let kd = self.kd.evaluate(si);
        if !kd.is_black() {
            bsdf.add(arena.alloc(LambertianReflection { r: kd }))
        }

        let ks = self.ks.evaluate(si);
        if !ks.is_black() {
            let fresnel = FresnelDielectric::new(1.5, 1.0);
            let mut rough = self.roughness.evaluate(si);
            if self.remap_roughness {
                rough = TrowbridgeReitzDistribution::roughness_to_alpha(rough);
            }
            let distribution = TrowbridgeReitzDistribution::new(rough, rough);
            let specular = MicrofacetReflection {
                r: ks,
                distribution,
                fresnel
            };
            bsdf.add(arena.alloc(specular))
        }
        bsdf
    }
}