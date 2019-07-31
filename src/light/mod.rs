use crate::{Transform, Point2f, Vec3f, Float};
use crate::interaction::HitPoint;
use crate::spectrum::Spectrum;

pub trait Light {
    const FLAGS: LightFlags;

    fn light_to_world(&self) -> &Transform;

    fn world_to_light(&self) -> &Transform;

    fn n_samples(&self) -> usize { 1 }

    fn sample_incident_radiance(&self, reference: &HitPoint, u: Point2f) -> LiSample;
}

pub struct LiSample {
    pub radiance: Spectrum,
    pub wi: Vec3f,
    pub pdf: Float,
    pub vis: VisibilityTester,
}

pub enum LightFlags {
    DeltaPosition, DeltaDirection, Area, Infinite
}

impl LightFlags {
    pub fn is_delta_light(&self) -> bool {
        match self {
            LightFlags::DeltaDirection | LightFlags::DeltaPosition => true,
            _ => false
        }
    }
}

pub struct VisibilityTester {

}