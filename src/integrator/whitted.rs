use crate::integrator::IntegratorRadiance;
use crate::scene::Scene;
use crate::sampler::Sampler;
use crate::RayDifferential;
use bumpalo::Bump;
use crate::spectrum::Spectrum;

pub struct WhittedIntegrator {
    pub max_depth: u16,
}

impl IntegratorRadiance for WhittedIntegrator {
    fn preprocess(&mut self, scene: &Scene, sampler: &Sampler) {
        unimplemented!()
    }

    fn incident_radiance(&self, ray: &mut RayDifferential, scene: &Scene, sampler: &Sampler, arena: &Bump, depth: u16) -> Spectrum {
        let radiance: Spectrum = Spectrum::new(0.0);

        match scene.intersect(&mut ray.ray) {
            None => {

            },

            Some(intersect) => {

            }

        }

        unimplemented!()
    }
}



