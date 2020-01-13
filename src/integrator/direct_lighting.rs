use crate::integrator::IntegratorRadiance;
use crate::sampler::Sampler;
use bumpalo::Bump;
use crate::{RayDifferential, SurfaceInteraction, Point2f, abs_dot, Float};
use crate::spectrum::{Spectrum};
use crate::scene::Scene;
use crate::material::TransportMode;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::BxDFType;
use crate::light::Light;
use crate::sampling::power_heuristic;

pub enum LightStrategy {

    /// Loops over all of the lights and takes a number of samples from each based on
    /// `Light::n_samples` and sums the result. This applies the Monte Carlo technique of splitting.
    UniformSampleAll,

    /// Takes a single sample from one of the lights, chosen at random.
    UniformSampleOne
}

pub struct DirectLightingIntegrator {
    pub strategy: LightStrategy,
    pub max_depth: u16,
    pub n_light_samples: Vec<usize>,
//    pub light_sample_ids:
}

impl DirectLightingIntegrator {

}

impl IntegratorRadiance for DirectLightingIntegrator {
    fn preprocess(&mut self, scene: &Scene, sampler: &mut dyn Sampler) {
        if let LightStrategy::UniformSampleAll = self.strategy {

            // Store the number of samples to be used for each light.
            // TODO: give each light an id? Currently just relies on consistent iteration order.
            self.n_light_samples = scene.lights.iter()
                .map(|light| sampler.round_count(light.n_samples()))
                .collect();

            for _ in 0..self.max_depth {
                for &n_samples in &self.n_light_samples {
                    sampler.request_2d_array(n_samples);
                    sampler.request_2d_array(n_samples);
                }
            }
        }
    }

    fn incident_radiance(&self, ray: &mut RayDifferential, scene: &Scene, sampler: &mut dyn Sampler, arena: &Bump, depth: u16) -> Spectrum {
        let mut radiance: Spectrum = Spectrum::uniform(0.0);

        match scene.intersect(&mut ray.ray) {
            None => {
                // get radiance of escaping ray
//                background(ray.ray.dir)
                Spectrum::uniform(0.0)
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

                    // Add emitted light if ray hit an area light source.
                    radiance += intersect.emitted_radiance(intersect.wo);

                    radiance += match self.strategy {
                        LightStrategy::UniformSampleAll => {
                            uniform_sample_all_lights(
                                &intersect,
                                &bsdf,
                                scene,
                                arena,
                                sampler,
                                &self.n_light_samples
                            )
                        },
                        LightStrategy::UniformSampleOne => {
                            uniform_sample_one_light(
                                &intersect,
                                &bsdf,
                                scene,
                                arena,
                                sampler,
                            )
                        }
                    };

                    if depth + 1 < self.max_depth {
                        radiance += self.specular_reflect(ray, &intersect, &bsdf, scene, sampler, arena, depth);
                        radiance += self.specular_transmit(ray, &intersect, &bsdf, scene, sampler, arena, depth);
                    }
                } else {
                    unimplemented!()
                }

                radiance
            }

        }
    }
}

fn uniform_sample_one_light(
    intersect: &SurfaceInteraction,
    bsdf: &Bsdf,
    scene: &Scene,
    arena: &Bump,
    sampler: &mut dyn Sampler,
) -> Spectrum {
    let n_lights = scene.lights.len();
    if n_lights == 0 { return Spectrum::uniform(0.0) }

    let light_num = (sampler.get_1d() * (n_lights as Float)).min((n_lights - 1) as Float) as usize;
    let light = scene.lights[light_num].as_ref();

    let u_light = sampler.get_2d();
    let u_scattering = sampler.get_2d();
    n_lights as Float * estimate_direct(bsdf, intersect, u_scattering, light, u_light, scene, arena)
}

fn uniform_sample_all_lights(
    intersect: &SurfaceInteraction,
    bsdf: &Bsdf,
    scene: &Scene,
    arena: &Bump,
    sampler: &mut dyn Sampler,
    n_light_samples: &[usize],
) -> Spectrum {
    unimplemented!()
//    scene.lights.iter().zip(n_light_samples).map(|(light, &n_samples)| {
//        // TODO: sampler return optional arrays
//        let u_light_array = sampler.get_2d_array(n_samples);
//        let u_scattering_array = sampler.get_2d_array(n_samples);
//
//        u_light_array.iter().zip(u_scattering_array)
//            .map(|(&u_light, &u_scattering)| {
//                estimate_direct(
//                    bsdf,
//                    intersect,
//                    u_scattering,
//                    *light,
//                    u_light,
//                    scene,
//                    arena,
////                    sampler, // TODO: ??? would be needed for volumes
//                )
//            }).sum::<Spectrum>() / (n_samples as Float)
//    }).sum()
}

fn estimate_direct(
    bsdf: &Bsdf,
    intersect: &SurfaceInteraction,
    u_scattering: Point2f,
    light: &dyn Light,
    u_light: Point2f,
    scene: &Scene,
    arena: &Bump,
//    sampler: &mut dyn Sampler,
) -> Spectrum {
    let bsdf_flags = BxDFType::all() & !BxDFType::SPECULAR;
    let mut radiance = Spectrum::uniform(0.0);

    let light_sample = light.sample_incident_radiance(&intersect.hit, u_light);

    if light_sample.pdf > 0.0 && !light_sample.radiance.is_black() {
        // Evaluate BSDF for light sampling strategy
        let f =
            bsdf.f(intersect.wo, light_sample.wi, bsdf_flags) *
            abs_dot(light_sample.wi, intersect.shading_n.0);

        let scattering_pdf = bsdf.pdf(intersect.wo, light_sample.wi, bsdf_flags);

        // If the BSDF would reflect the radiance from this light, only then trace a
        // shadow ray to see if the light is unoccluded
        if !f.is_black() && light_sample.vis.unoccluded(scene) {
            radiance += if light.flags().is_delta_light() {
                f * light_sample.radiance / light_sample.pdf
            } else {
                let weight = power_heuristic(1, light_sample.pdf, 1, scattering_pdf);
                f * light_sample.radiance * weight / light_sample.pdf
            }
        }
    }

    if !light.flags().is_delta_light() {
        let scatter = bsdf.sample_f(intersect.wo, u_scattering, bsdf_flags);
        if let Some(scatter) = scatter {
            let f = scatter.f * abs_dot(scatter.wi, intersect.shading_n.0);
            let sampled_specular = scatter.sampled_type.contains(BxDFType::SPECULAR);

            let weight = if sampled_specular {
                1.0
            } else {
                let light_pdf = light.pdf_incident_radiance(&intersect.hit, scatter.wi);
                if light_pdf == 0.0 {
                    return radiance;
                }
                power_heuristic(1, scatter.pdf, 1, light_pdf)
            };
            let mut ray = intersect.hit.spawn_ray(scatter.wi);

            // TODO: Specialized bvh query for testing ray between two known objects?
            let si = scene.intersect(&mut ray);

            let incident_radiance = if let Some(si) = si {
                si.primitive.unwrap().area_light()
                    // TODO: make sure this actually works
                    .filter(|l| std::ptr::eq(l.as_light(), light))
                    // TODO: just call emitted on light?
                    .map_or(Spectrum::uniform(0.0), |_| si.emitted_radiance(-scatter.wi))
            } else {
                // TODO: how to get differentials
                light.environment_emitted_radiance(&RayDifferential { ray, diff: None })
            };

            if !incident_radiance.is_black() {
                radiance += f * incident_radiance * weight / scatter.pdf
            }

        }
    }

    radiance
}