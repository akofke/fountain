use crate::texture::{Texture, ConstantTexture, TextureRef};
use crate::spectrum::Spectrum;
use std::sync::Arc;
use crate::{Float, SurfaceInteraction};
use crate::material::{Material, TransportMode};
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::{SpecularReflection, SpecularTransmission, MicrofacetReflection, MicrofacetTransmission};
use crate::fresnel::FresnelDielectric;
use crate::reflection::microfacet::TrowbridgeReitzDistribution;

pub struct GlassMaterial {
    reflectance: Arc<dyn Texture<Output = Spectrum>>,
    transmittance: Arc<dyn Texture<Output = Spectrum>>,
    u_roughness: TextureRef<Float>,
    v_roughness: TextureRef<Float>,
    eta: Arc<dyn Texture<Output = Float>>,
    remap_roughness: bool,
}

impl GlassMaterial {
    pub fn new(
        kr: TextureRef<Spectrum>,
        kt: TextureRef<Spectrum>,
        u_roughness: TextureRef<Float>,
        v_roughness: TextureRef<Float>,
        eta: TextureRef<Float>,
        remap_roughness: bool
    ) -> Self {
        Self {
            reflectance: kr,
            transmittance: kt,
            u_roughness,
            v_roughness,
            eta,
            remap_roughness,
        }
    }
    pub fn constant(kr: Spectrum, kt: Spectrum, eta: Float) -> Self {
        Self {
            reflectance: Arc::new(ConstantTexture(kr)),
            transmittance: Arc::new(ConstantTexture(kt)),
            u_roughness: Arc::new(ConstantTexture(0.0)),
            v_roughness: Arc::new(ConstantTexture(0.0)),
            eta: Arc::new(ConstantTexture(eta)),
            remap_roughness: false
        }
    }
}

impl Material for GlassMaterial {
    fn compute_scattering_functions<'a>(&self, si: &SurfaceInteraction, arena: &'a Bump, mode: TransportMode, allow_multiple_lobes: bool) -> Bsdf<'a> {
        let eta = self.eta.evaluate(si);
        let r = self.reflectance.evaluate(si).clamp_positive();
        let t = self.transmittance.evaluate(si).clamp_positive();
        let mut u_rough = self.u_roughness.evaluate(si);
        let mut v_rough = self.v_roughness.evaluate(si);
        if self.remap_roughness {
            u_rough = TrowbridgeReitzDistribution::roughness_to_alpha(u_rough);
            v_rough = TrowbridgeReitzDistribution::roughness_to_alpha(v_rough);
        }
        let mut bsdf = Bsdf::new(si, eta);

        let is_specular = u_rough == 0.0 && v_rough == 0.0;

        if is_specular && allow_multiple_lobes {
            todo!("FresnelSpecular")
        } else {
            if !r.is_black() {
                let fresnel = FresnelDielectric::new(1.0, eta);
                if is_specular {
                    let reflection = arena.alloc(SpecularReflection::new(r, fresnel));
                    bsdf.add(reflection);
                } else {
                    let distribution = TrowbridgeReitzDistribution::new(u_rough, v_rough);
                    let reflection = arena.alloc(MicrofacetReflection::new(r, distribution, fresnel));
                    bsdf.add(reflection);
                }
            }

            if !t.is_black() {
                if is_specular {
                    let transmission = arena.alloc(SpecularTransmission::new(t, 1.0, eta, mode));
                    bsdf.add(transmission);
                } else {
                    let distribution = TrowbridgeReitzDistribution::new(u_rough, v_rough);
                    let transmission = arena.alloc(MicrofacetTransmission::new(t, distribution, 1.0, eta, mode));
                    bsdf.add(transmission);
                }
            }
        }
        bsdf
    }
}