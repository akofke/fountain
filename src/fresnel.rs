use crate::Float;
use crate::spectrum::Spectrum;

fn fresnel_dielectric(cos_theta_i: Float, mut eta_i: Float, mut eta_t: Float) -> Float {
    let cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);
    let entering = cos_theta_i > 1.0;
    if !entering {
        std::mem::swap(&mut eta_i, &mut eta_t)
    }

    // compute cos_theta_t using snell's law
    let sin_theta_t = Float::sqrt((1.0 - cos_theta_i * cos_theta_i).max(0.0));
    let sin_theta_t = eta_i / eta_t * sin_theta_t;
    if sin_theta_t >= 1.0 { return 1.0 } // total internal reflection
    let cos_theta_t = Float::sqrt((1.0 - sin_theta_t * sin_theta_t).max(0.0));

    let r_parallel = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t)) / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
    let r_perp =     ((eta_i * cos_theta_i) - (eta_t * cos_theta_t)) / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));

    (r_parallel * r_parallel + r_perp * r_perp) / 2.0
}

#[allow(non_snake_case)]
fn fresnel_conductor(cos_theta_i: Float, eta_i: Spectrum, eta_t: Spectrum, k: Spectrum) -> Spectrum {
    let cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);
    let eta = eta_t / eta_i;
    let eta_k = k / eta_i;

    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i2 = 1.0 - cos_theta_i2;
    let eta2 = eta * eta;
    let eta_k2 = eta_k * eta_k;

    let t0 = eta2 - eta_k2 - sin_theta_i2;
    let a2plusb2 = (t0 * t0 + 4.0 * eta2 * eta_k2).sqrt();
    let t1 = a2plusb2 + cos_theta_i2;
    let a = (0.5 * (cos_theta_i + t0)).sqrt();
    let t2 = 2.0 * cos_theta_i * a;
    let Rs = (t1 - t2) / (t1 + t2);

    let t3 = cos_theta_i2 * a2plusb2 + sin_theta_i2 * sin_theta_i2;
    let t4 = t2 * sin_theta_i2;
    let Rp = Rs * (t3 - t4) / (t3 + t4);

    0.5 * (Rp + Rs)

}

pub trait Fresnel {

    /// Given the cosine of the angle made by the incoming direction and the surface normal,
    /// returns the amount of light reflected by the surface.
    fn evaluate(&self, cos_i: Float) -> Spectrum;
}

pub struct FresnelConductor {
    /// incident index of refraction
    eta_i: Spectrum,

    /// transmitted index of refraction
    eta_t: Spectrum,

    /// absorption coefficient
    k: Spectrum,
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cos_i: Float) -> Spectrum {
        fresnel_conductor(cos_i.abs(), self.eta_i, self.eta_t, self.k)
    }
}

pub struct FresnelDielectric {
    /// incident index of refraction
    eta_i: Float,

    /// transmitted index of refraction
    eta_t: Float,
}

impl FresnelDielectric {
    pub fn new(eta_i: Float, eta_t: Float) -> Self {
        Self { eta_i, eta_t }
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cos_i: Float) -> Spectrum {
        Spectrum::new(fresnel_dielectric(cos_i, self.eta_i, self.eta_t))
    }
}

