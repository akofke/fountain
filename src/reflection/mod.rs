use bitflags::bitflags;
use crate::{Vec3f, Point2f, Float, Normal3};
use crate::spectrum::Spectrum;
use crate::fresnel::{Fresnel, FresnelDielectric};
use crate::material::TransportMode;
use cgmath::InnerSpace;
use crate::sampling::cosine_sample_hemisphere;
use bitflags::_core::fmt::Debug;

pub mod bsdf;

bitflags! {
    pub struct BxDFType: u8 {
        const REFLECTION = 1;
        const TRANSMISSION = 1 << 1;
        const DIFFUSE = 1 << 2;
        const GLOSSY = 1 << 3;
        const SPECULAR = 1 << 4;
    }
}

fn cos_theta(w: Vec3f) -> Float { w.z }
fn cos2_theta(w: Vec3f) -> Float { w.z * w.z }
fn abs_cos_theta(w: Vec3f) -> Float { w.z.abs() }

fn sin2_theta(w: Vec3f) -> Float {
    Float::max(0.0, 1.0 - cos2_theta(w))
}

fn sin_theta(w: Vec3f) -> Float {
    sin2_theta(w).sqrt()
}

fn tan_theta(w: Vec3f) -> Float {
    sin_theta(w) / cos_theta(w)
}

fn tan2_theta(w: Vec3f) -> Float {
    sin2_theta(w) / cos2_theta(w)
}

pub fn refract(wi: Vec3f, n: Normal3, eta: Float) -> Option<Vec3f> {
    let cos_theta_i = n.dot(wi);
    let sin2_theta_i = Float::max(0.0, 1.0 - cos_theta_i * cos_theta_i);
    let sin2_theta_t = eta * eta * sin2_theta_i;
    if sin2_theta_t >= 1.0 { return None }
    let cos_theta_t = Float::sqrt(1.0 - sin2_theta_t);
    let wt = eta * -wi + (eta * cos_theta_i - cos_theta_t) * n.0;
    Some(wt)
}

pub fn same_hemisphere(v1: Vec3f, v2: Vec3f) -> bool {
    v1.z.is_sign_positive() == v2.z.is_sign_positive()
}

#[derive(Clone, Copy)]
pub struct ScatterSample {
    pub f: Spectrum,
    pub wi: Vec3f,
    pub pdf: Float,
    pub sampled_type: BxDFType
}

pub trait BxDF: Debug {

    fn matches_flags(&self, t: BxDFType) -> bool {
        t.contains(self.get_type())
    }

    fn get_type(&self) -> BxDFType;

    /// Returns the value of the distribution function for the given pair of directions.
    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum;

    /// Computes the direction of incident light wi given an outgoing direction wo as well
    /// as the value of the BxDF for the pair of directions. TODO other uses...
    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> Option<ScatterSample>;

    fn pdf(&self, wo: Vec3f, wi: Vec3f) -> Float;

}

// TODO: better name
pub trait DefaultSampleF: Debug {
    fn get_type(&self) -> BxDFType;

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum;
}

impl<T> BxDF for T where T: DefaultSampleF {
    fn get_type(&self) -> BxDFType {
        <Self as DefaultSampleF>::get_type(self)
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        <Self as DefaultSampleF>::f(self, wo, wi)
    }

    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> Option<ScatterSample> {
        let mut wi = cosine_sample_hemisphere(sample);
        // flip direction if wo is on the opposite hemisphere
        if wo.z < 0.0 { wi.z *= -1.0; }
        let pdf = self.pdf(wo, wi);
        let f = self.f(wo, wi);
        Some(ScatterSample { f, wi, pdf, sampled_type: self.get_type() })
    }

    fn pdf(&self, wo: Vec3f, wi: Vec3f) -> Float {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * std::f32::consts::FRAC_1_PI
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct LambertianReflection {
    pub r: Spectrum,
}

impl DefaultSampleF for LambertianReflection {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::DIFFUSE
    }

    fn f(&self, _wo: Vec3f, _wi: Vec3f) -> Spectrum {
        self.r * std::f32::consts::FRAC_1_PI
    }
}

#[derive(Debug)]
pub struct SpecularReflection<F: Fresnel> {
    r: Spectrum,
    fresnel: F
}

impl<F: Fresnel> SpecularReflection<F> {
    pub fn new(r: Spectrum, fresnel: F) -> Self {
        Self {r, fresnel}
    }
}

impl<F: Fresnel> BxDF for SpecularReflection<F> {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::SPECULAR
    }

    fn f(&self, _wo: Vec3f, _wi: Vec3f) -> Spectrum {
        Spectrum::new(0.0)
    }

    fn sample_f(&self, wo: Vec3f, _sample: Point2f) -> Option<ScatterSample> {
        let wi = Vec3f::new(-wo.x, -wo.y, wo.z);
        let pdf = 1.0f32;


        let reflected = self.fresnel.evaluate(cos_theta(wi)) * self.r / abs_cos_theta(wi);
        Some(ScatterSample{f: reflected, wi, pdf, sampled_type: self.get_type()})
    }

    fn pdf(&self, wo: Vec3f, wi: Vec3f) -> Float {
        0.0
    }
}

#[derive(Debug)]
pub struct SpecularTransmission {
    t: Spectrum,
    eta_a: Float,
    eta_b: Float,
    fresnel: FresnelDielectric,
    mode: TransportMode,
}

impl SpecularTransmission {
    pub fn new(t: Spectrum, eta_a: Float, eta_b: Float, mode: TransportMode) -> Self {
        Self {
            t, eta_a, eta_b, mode, fresnel: FresnelDielectric::new(eta_a, eta_b)
        }
    }
}

impl BxDF for SpecularTransmission {
    fn get_type(&self) -> BxDFType {
        BxDFType::TRANSMISSION | BxDFType::SPECULAR
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        Spectrum::new(0.0)
    }

    fn sample_f(&self, wo: Vec3f, _sample: Point2f) -> Option<ScatterSample> {
        let entering = cos_theta(wo) > 0.0;
        let eta_i = if entering { self.eta_a } else { self.eta_b };
        let eta_t = if entering { self.eta_b } else { self.eta_a };

        let wi = refract(
            wo,
            Normal3::new(0.0, 0.0, 1.0).faceforward(wo),
            eta_i / eta_t
        )?;

        let pdf = 1.0f32;
        let ft = self.t * (Spectrum::new(1.0) - self.fresnel.evaluate(cos_theta(wi)));
        Some(ScatterSample {
            f: ft / abs_cos_theta(wi),
            wi,
            pdf,
            sampled_type: self.get_type()
        })
    }

    fn pdf(&self, _wo: Vec3f, _wi: Vec3f) -> Float {
        0.0
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ray, SurfaceInteraction, Transform};
    use crate::shapes::sphere::Sphere;
    use crate::shapes::Shape;

//    fn get_test_surface_interaction(ray: &Ray) -> SurfaceInteraction {
//        let sphere = Sphere::whole(&Transform::IDENTITY, &Transform::IDENTITY, 1.0);
//    }

    #[test]
    fn test_specular_reflection() {

    }
}

