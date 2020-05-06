use bitflags::bitflags;
use crate::{Vec3f, Point2f, Float, Normal3, faceforward, abs_dot};
use crate::spectrum::Spectrum;
use crate::fresnel::{Fresnel, FresnelDielectric};
use crate::material::TransportMode;
use cgmath::{InnerSpace, Rad};
use crate::sampling::cosine_sample_hemisphere;
use std::fmt::Debug;
use crate::reflection::microfacet::MicrofacetDistribution;

pub mod bsdf;
pub mod microfacet;

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

fn cos_phi(w: Vec3f) -> Float {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x / sin_theta).clamp(-1.0, 1.0)
    }
}

fn sin_phi(w: Vec3f) -> Float {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        0.0
    } else {
        (w.y / sin_theta).clamp(-1.0, 1.0)
    }
}

fn cos2_phi(w: Vec3f) -> Float {
    cos_phi(w) * cos_phi(w)
}

fn sin2_phi(w: Vec3f) -> Float {
    sin_phi(w) * sin_phi(w)
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

pub fn reflect(wo: Vec3f, n: Vec3f) -> Vec3f {
    -wo + 2.0 * wo.dot(n) * n
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

pub trait BxDF {

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

// TODO: better name - CosineSampledBxDF?
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
        Spectrum::uniform(0.0)
    }

    fn sample_f(&self, wo: Vec3f, _sample: Point2f) -> Option<ScatterSample> {
        let wi = Vec3f::new(-wo.x, -wo.y, wo.z);
        let pdf = 1.0f32;


        let reflected = self.fresnel.evaluate(cos_theta(wi)) * self.r / abs_cos_theta(wi);
        Some(ScatterSample{f: reflected, wi, pdf, sampled_type: self.get_type()})
    }

    fn pdf(&self, _wo: Vec3f, _wi: Vec3f) -> Float {
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

    fn f(&self, _wo: Vec3f, _wi: Vec3f) -> Spectrum {
        Spectrum::uniform(0.0)
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
        let ft = self.t * (Spectrum::uniform(1.0) - self.fresnel.evaluate(cos_theta(wi)));
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

#[derive(Debug)]
pub struct OrenNayar {
    pub r: Spectrum,
    pub a: Float,
    pub b: Float,
}

impl OrenNayar {
    pub fn new(r: Spectrum, sigma: impl Into<Rad<Float>>) -> Self {
        let sigma = sigma.into().0;
        let sigma2 = sigma * sigma;
        let a = 1.0 - (sigma2 / (2.0 * (sigma2 + 0.33)));
        let b = 0.45 * sigma2 / (sigma2 + 0.09);
        OrenNayar { r, a, b }
    }
}

impl DefaultSampleF for OrenNayar {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::DIFFUSE
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        let sin_theta_i = sin_theta(wi);
        let sin_theta_o = sin_theta(wo);
        // compute cosine term of Oren-Nayar model
        let max_cos = if sin_theta_i > 1.0e-4 && sin_theta_o > 1.0e-4 {
            let sin_phi_i = sin_phi(wi);
            let cos_phi_i = cos_phi(wi);
            let sin_phi_o = sin_phi(wo);
            let cos_phi_o = cos_phi(wo);
            let d_cos = cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o;
            Float::max(0.0, d_cos)
        } else {
            0.0
        };

        let (sin_alpha, tan_beta) = if abs_cos_theta(wi) > abs_cos_theta(wo) {
            (sin_theta_o, sin_theta_i / abs_cos_theta(wi))
        } else {
            (sin_theta_i, sin_theta_o / abs_cos_theta(wo))
        };

        self.r * crate::consts::FRAC_1_PI * (self.a + (self.b * max_cos * sin_alpha * tan_beta))
    }
}

/// A general microfacet-based BRDF using the Torrance-Sparrow model.
#[derive(Debug)]
pub struct MicrofacetReflection<D: MicrofacetDistribution, F: Fresnel> {
    pub r: Spectrum,
    pub distribution: D,
    pub fresnel: F,
}

impl<D: MicrofacetDistribution, F: Fresnel> MicrofacetReflection<D, F> {
    pub fn new(r: Spectrum, distribution: D, fresnel: F) -> Self {
        MicrofacetReflection { r, distribution, fresnel }
    }
}

impl<D: MicrofacetDistribution, F: Fresnel> BxDF for MicrofacetReflection<D, F> {
    fn get_type(&self) -> BxDFType {
        BxDFType::REFLECTION | BxDFType::GLOSSY
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i= abs_cos_theta(wi);
        let wh = wi + wo;

        // handle degenerate cases
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 || (wh == Vec3f::new(0.0, 0.0, 0.0)) {
            return Spectrum::uniform(0.0)
        }
        let wh = wh.normalize();

        // For the Fresnel call, make sure that wh is in the same hemisphere as the surface
        // normal so total internal reflection is handled correctly.
        let f = self.fresnel.evaluate(
            wi.dot(faceforward(wh, Vec3f::new(0.0, 0.0, 1.0))));

        self.r * self.distribution.d(wh) * self.distribution.g(wo, wi) * f
            / (4.0 * cos_theta_i * cos_theta_o)
    }

    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> Option<ScatterSample> {
        let wh = self.distribution.sample_wh(wo, sample);
        let wi = reflect(wo, wh);
        if !same_hemisphere(wo, wi) {
            return None;
        }

        let pdf = self.distribution.pdf(wo, wh) / (4.0 * wo.dot(wh));
        ScatterSample {
            f: self.f(wo, wi),
            wi,
            pdf,
            sampled_type: self.get_type()
        }.into()
    }

    fn pdf(&self, wo: Vec3f, wi: Vec3f) -> Float {
        if !same_hemisphere(wo, wi) {
            return 0.0
        }
        let wh = (wo + wi).normalize();
        self.distribution.pdf(wo, wh) / (4.0 * wo.dot(wh))
    }
}

pub struct MicrofacetTransmission<D: MicrofacetDistribution> {
    pub t: Spectrum,
    pub distribution: D,
    pub eta_a: Float,
    pub eta_b: Float,
    pub fresnel: FresnelDielectric,
    pub mode: TransportMode,
}

impl<D: MicrofacetDistribution> MicrofacetTransmission<D> {
    pub fn new(t: Spectrum, distribution: D, eta_a: Float, eta_b: Float, mode: TransportMode) -> Self {
        MicrofacetTransmission { t, distribution, eta_a, eta_b, fresnel: FresnelDielectric::new(eta_a, eta_b), mode }
    }

    fn get_eta(&self, wo: Vec3f) -> Float {
        if cos_theta(wo) > 0.0 { self.eta_b / self.eta_a } else { self.eta_a / self.eta_b }
    }
}

impl<D: MicrofacetDistribution> BxDF for MicrofacetTransmission<D> {
    fn get_type(&self) -> BxDFType {
        BxDFType::TRANSMISSION | BxDFType::GLOSSY
    }

    fn f(&self, wo: Vec3f, wi: Vec3f) -> Spectrum {
        if same_hemisphere(wo, wi) {
            return Spectrum::uniform(0.0);
        }
        let cos_theta_o = cos_theta(wo);
        let cos_theta_i  = cos_theta(wi);
        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Spectrum::uniform(0.0);
        }

        let eta = self.get_eta(wo);
        let wh = (wo + wi * eta).normalize();
        let wh = if wh.z < 0.0 { -wh } else { wh };
        let f = self.fresnel.evaluate(wo.dot(wh));
        let sqrt_denom = wo.dot(wh) + eta * wi.dot(wh);
        let factor = if self.mode == TransportMode::Radiance { 1.0 / eta } else { 1.0 };
        (Spectrum::uniform(1.0) - f) * self.t *
            Float::abs(self.distribution.d(wh) * self.distribution.g(wo, wi) * sq!(eta) * abs_dot(wi, wh) * abs_dot(wo, wh) * sq!(factor)
            / (cos_theta_i * cos_theta_o * sq!(sqrt_denom)))
    }

    fn sample_f(&self, wo: Vec3f, sample: Point2f) -> Option<ScatterSample> {
        if wo.z == 0.0 {
            return None;
        }
        let wh = self.distribution.sample_wh(wo, sample);
        if wo.dot(wh) < 0.0 {
            return None;
        }
        let eta = self.get_eta(-wo); // NOTE: this inverts the eta fraction
        if let Some(wi) = refract(wo, Normal3(wh), eta) {
            ScatterSample {
                f: self.f(wo, wi),
                wi,
                pdf: self.pdf(wo, wi),
                sampled_type: self.get_type()
            }.into()
        } else {
            None
        }
    }

    fn pdf(&self, wo: Vec3f, wi: Vec3f) -> Float {
        if same_hemisphere(wo, wi) {
            return 0.0
        }
        let eta = self.get_eta(wo);
        let wh = (wo + wi * eta).normalize();
        let sqrt_denom = wo.dot(wh) + eta * wi.dot(wh);
        let dwh_dwi = Float::abs((sq!(eta) * wi.dot(wh)) / sq!(sqrt_denom));
        self.distribution.pdf(wo, wh) * dwh_dwi
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

