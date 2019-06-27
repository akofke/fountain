use bitflags::bitflags;
use crate::{Vec3f, Point2f};
use crate::spectrum::Spectrum;
use crate::fresnel::Fresnel;

bitflags! {
    pub struct BxDFType: u8 {
        const REFLECTION = 1 << 0;
        const TRANSMISSION = 1 << 1;
        const DIFFUSE = 1 << 2;
        const GLOSSY = 1 << 3;
        const SPECULAR = 1 << 4;
    }
}

pub trait BxDF {

    fn matches_flags(&self, t: BxDFType) -> bool {
        unimplemented!()
    }

    fn get_type(&self) -> BxDFType;

    /// Returns the value of the distribution function for the given pair of directions.
    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum;

    /// Computes the direction of incident light wi given an outgoing direction wo as well
    /// as the value of the BxDF for the pair of directions. TODO other uses...
    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> ();

}

pub struct LambertianReflection {
    r: Spectrum,
}

impl BxDF for LambertianReflection {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::DIFFUSE
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        unimplemented!()
    }

    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> () {
        unimplemented!()
    }
}

pub struct SpecularReflection<F: Fresnel> {
    r: Spectrum,
    fresnel: F
}

