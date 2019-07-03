use crate::interaction::SurfaceInteraction;
use bumpalo::Bump;
use crate::reflection::bsdf::Bsdf;

pub mod matte;

pub enum TransportMode {
    Radiance,
    Importance,
}

pub trait Material {
    fn compute_scattering_functions(
        &self,
        si: &SurfaceInteraction,
        arena: &Bump,
        mode: TransportMode,
        allow_multiple_lobes: bool
    ) -> Bsdf;
}