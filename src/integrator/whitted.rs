use crate::integrator::IntegratorRadiance;
use crate::scene::Scene;
use crate::sampler::Sampler;
use crate::RayDifferential;
use bumpalo::Bump;
use crate::spectrum::Spectrum;
use crate::material::TransportMode;

pub struct WhittedIntegrator {
    pub max_depth: u16,
}

impl IntegratorRadiance for WhittedIntegrator {
    fn preprocess(&mut self, scene: &Scene, sampler: &dyn Sampler) {
        unimplemented!()
    }

    fn incident_radiance(&self, ray: &mut RayDifferential, scene: &Scene, sampler: &mut dyn Sampler, arena: &Bump, depth: u16) -> Spectrum {
        let mut radiance: Spectrum = Spectrum::new(0.0);

        match scene.intersect(&mut ray.ray) {
            None => {
                // get radiance of escaping ray
                Spectrum::new(1.0)
            },

            Some(mut intersect) => {
                let n = intersect.shading_n;
                let wo = intersect.wo;

                let bsdf = intersect.compute_scattering_functions(
                    ray,
                    arena,
                    false,
                    TransportMode::Radiance
                );

                if let Some(bsdf) = bsdf {

                    if depth + 1 < self.max_depth {
                        radiance += self.specular_reflect(ray, &intersect, &bsdf, scene, sampler, arena, depth);
                    }
                } else {
                    unimplemented!()
                }

                radiance
            }

        }
    }
}



