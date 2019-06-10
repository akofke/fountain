
pub trait SamplerIntegrator {
    /// Returns the radiance (Li) arriving at the origin of the given ray
    fn radiance(&self /* ... */ ); // -> Spectrum
}

pub struct WhittedIntegrator {

}