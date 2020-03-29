use crate::{Vec3f, Float, Point2f, spherical_direction};
use crate::reflection::{tan2_theta, cos2_theta, cos2_phi, sin2_phi, tan_theta, same_hemisphere, abs_cos_theta};
use crate::consts::{PI, FRAC_PI_2};

pub trait MicrofacetDistribution {
    /// Find the differential area of microfacets oriented with the given normal vector `w`
    fn d(&self, wh: Vec3f) -> Float;

    /// The Smith masking-shadowing function, giving the fraction of microfacets with normal `wh`
    /// that are visible from direction `w`. In most cases the probability a microfacet is visible
    /// is independent from its orientation so this function only depends on `w`.
    fn g1(&self, w: Vec3f) -> Float {
        1.0 / (1.0 + self.lambda(w))
    }

    /// Measures invisible masked microfacet area per visible microfacet area.
    fn lambda(&self, w: Vec3f) -> Float;

    /// Gives the fraction of microfacets in a differential area that are visible from both
    /// directions `wo` and `wi`.
    fn g(&self, wo: Vec3f, wi: Vec3f) -> Float {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    /// Sample from the distribution of normal vectors visible from direction `wo`.
    fn sample_wh(&self, wo: Vec3f, u: Point2f) -> Vec3f;

    fn pdf(&self, wo: Vec3f, wh: Vec3f) -> Float {
        // TODO: change when sampling visible area
        self.d(wh) * abs_cos_theta(wh)
    }
}

pub struct BeckmannDistribution {
    alpha_x: Float,
    alpha_y: Float,
}

impl BeckmannDistribution {
    pub fn roughness_to_alpha(roughness: Float) -> Float {
        let rough = roughness.max(1.0e-3);
        let x = rough.ln();
        1.62142 + 0.819955 * x + 0.1734 * x * x +
            0.0171201 * x * x * x + 0.000640711 * x * x * x * x
    }

    pub fn new(alpha_x: Float, alpha_y: Float) -> Self {
        BeckmannDistribution { alpha_x, alpha_y }
    }
}

impl MicrofacetDistribution for BeckmannDistribution {
    fn d(&self, wh: Vec3f) -> Float {
        let tan2_theta = tan2_theta(wh);
        if tan2_theta.is_infinite() {
            return 0.0
        }

        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
        Float::exp(
            -tan2_theta
                * (cos2_phi(wh) / (self.alpha_x * self.alpha_x) + sin2_phi(wh) / (self.alpha_y * self.alpha_y))
                / (crate::consts::PI * self.alpha_x * self.alpha_y * cos4_theta)
        )
    }

    fn lambda(&self, w: Vec3f) -> Float {
        let abs_tan_theta = tan_theta(w).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0
        }

        // compute alpha for direction w
        let alpha = Float::sqrt(cos2_phi(w) * self.alpha_x * self.alpha_x +
            sin2_phi(w) * self.alpha_y * self.alpha_y);
        let a = 1.0 / (alpha * abs_tan_theta);
        if a > 1.6 {
            0.0
        } else {
            (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
        }
    }

    fn sample_wh(&self, wo: Vec3f, u: Point2f) -> Vec3f {
        // TODO: Sample from visible distribution!

        // Sample from full distribution of normals:

        let (tan2_theta, phi) = if self.alpha_x == self.alpha_y {
            let log_sample = Float::ln(1.0 - u[0]);
            debug_assert!(log_sample.is_finite());
            (-self.alpha_x * self.alpha_x * log_sample, u[1] * 2.0 * PI)
        } else {
            let log_sample = Float::ln(1.0 - u[0]);
            debug_assert!(log_sample.is_finite());
            let mut phi = Float::atan(self.alpha_y / self.alpha_x * Float::tan(2.0 * PI * u[1] + FRAC_PI_2));
            if u[1] > 0.5 {
                phi += PI;
            }
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            let tan2_theta = -log_sample / (sq!(cos_phi) / sq!(self.alpha_x) + sq!(sin_phi) / sq!(self.alpha_y));
            (tan2_theta, phi)
        };

        // map sampled angles to normal direction wh
        let cos_theta = 1.0 / (1.0 + tan2_theta).sqrt();
        let sin_theta = Float::sqrt(Float::max(0.0, 1.0 - sq!(cos_theta)));
        let wh = spherical_direction(sin_theta, cos_theta, phi);
        if same_hemisphere(wo, wh) {
            wh
        } else {
            -wh
        }
    }
}

/// Also known as GGX
pub struct TrowbridgeReitzDistribution {
    alpha_x: Float,
    alpha_y: Float,
}

impl TrowbridgeReitzDistribution {
    pub fn roughness_to_alpha(roughness: Float) -> Float {
        BeckmannDistribution::roughness_to_alpha(roughness)
    }

    pub fn new(alpha_x: Float, alpha_y: Float) -> Self {
        TrowbridgeReitzDistribution { alpha_x, alpha_y }
    }
}

impl MicrofacetDistribution for TrowbridgeReitzDistribution {
    fn d(&self, wh: Vec3f) -> Float {
        let tan2_theta = tan2_theta(wh);
        if tan2_theta.is_infinite() {
            return 0.0
        }

        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
        let e =
            (cos2_phi(wh) / (self.alpha_x * self.alpha_x) + sin2_phi(wh) / (self.alpha_y * self.alpha_y))
                * tan2_theta;
        1.0 / (crate::consts::PI * self.alpha_x * self.alpha_y * cos4_theta * (1.0 + e) * (1.0 + e))
    }

    fn lambda(&self, w: Vec3f) -> Float {
        let abs_tan_theta = tan_theta(w).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0
        }

        // compute alpha for direction w
        let alpha = Float::sqrt(cos2_phi(w) * self.alpha_x * self.alpha_x +
            sin2_phi(w) * self.alpha_y * self.alpha_y);

        let alpha2_tan2_theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);
        (-1.0 + Float::sqrt(1.0 + alpha2_tan2_theta)) / 2.0
    }

    fn sample_wh(&self, wo: Vec3f, u: Point2f) -> Vec3f {
        // TODO: Sample visible area!

        let (cos_theta, phi) = if self.alpha_x == self.alpha_y {
            let tan_theta2 = sq!(self.alpha_x) * u[0] / (1.0 - u[0]);
            (1.0 / Float::sqrt(1.0 + tan_theta2), 2.0 * PI * u[1])
        } else {
            let mut phi = Float::atan(self.alpha_y / self.alpha_x * Float::tan(2.0 * PI * u[1] + 0.5 * PI));
            if u[1] > 0.5 {
                phi += PI;
            }
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            let alpha2 = 1.0 / (sq!(cos_phi) / sq!(self.alpha_x) + sq!(sin_phi) / sq!(self.alpha_y));
            let tan_theta2 = alpha2 * u[0] / (1.0 - u[0]);
            (1.0 / Float::sqrt(1.0 + tan_theta2), phi)
        };
        let sin_theta = Float::sqrt(Float::max(0.0, 1.0 - sq!(cos_theta)));
        let wh = spherical_direction(sin_theta, cos_theta, phi);
        if same_hemisphere(wo, wh) {
            wh
        } else {
            -wh
        }
    }
}