use crate::texture::TextureRef;
use crate::spectrum::Spectrum;
use crate::{Float, SurfaceInteraction};
use crate::material::{Material, TransportMode};
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::microfacet::TrowbridgeReitzDistribution;
use crate::reflection::MicrofacetReflection;
use crate::fresnel::FresnelConductor;

pub enum RoughnessTex {
    Anisotropic {
        u_rough: TextureRef<Float>,
        v_rough: TextureRef<Float>,
    },
    Isotropic(TextureRef<Float>)
}

pub struct MetalMaterial {
    /// Index of refraction
    eta: TextureRef<Spectrum>,

    /// Absorption coefficient
    k: TextureRef<Spectrum>,

    roughness: RoughnessTex,

    remap_roughness: bool,
}

impl MetalMaterial {
    pub fn new(eta: TextureRef<Spectrum>, k: TextureRef<Spectrum>, roughness: RoughnessTex, remap_roughness: bool) -> Self {
        MetalMaterial { eta, k, roughness, remap_roughness }
    }
}

impl Material for MetalMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let (u_rough, v_rough) = match &self.roughness {
            RoughnessTex::Anisotropic { u_rough, v_rough} => {
                (u_rough.evaluate(si), v_rough.evaluate(si))
            },
            RoughnessTex::Isotropic(rough) => {
                let r = rough.evaluate(si);
                (r, r)
            }
        };
        let (u_rough, v_rough) = if self.remap_roughness {
            (TrowbridgeReitzDistribution::roughness_to_alpha(u_rough), TrowbridgeReitzDistribution::roughness_to_alpha(v_rough))
        } else { (u_rough, v_rough) };
        let distribution = TrowbridgeReitzDistribution::new(u_rough, v_rough);
        let fresnel = FresnelConductor {
            eta_i: Spectrum::uniform(1.0),
            eta_t: self.eta.evaluate(si),
            k: self.k.evaluate(si),
        };
        let mut bsdf = Bsdf::new(si, 1.0);
        let bxdf = MicrofacetReflection {
            r: Spectrum::uniform(1.0),
            distribution,
            fresnel,
        };
        bsdf.add(arena.alloc(bxdf));
        bsdf
    }
}