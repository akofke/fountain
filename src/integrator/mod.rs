use bumpalo::Bump;
use cgmath::InnerSpace;
use rayon::prelude::*;

use crate::{abs_dot, Bounds2i, Differential, Float, RayDifferential, SurfaceInteraction, Point2f};
use crate::camera::Camera;
use crate::film::Film;
use crate::filter::BoxFilter;
use crate::reflection::bsdf::Bsdf;
use crate::reflection::BxDFType;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::{Spectrum};
use std::sync::Arc;
use crate::light::Light;
use crate::sampling::power_heuristic;

pub mod whitted;
pub mod direct_lighting;
pub mod path;


pub struct SamplerIntegrator<R: IntegratorRadiance> {
    pub camera: Box<dyn Camera>,
    pub radiance: R,
}

pub trait IntegratorRadiance: Sync + Send {
    fn preprocess(&mut self, scene: &Scene, sampler: &mut dyn Sampler);

    fn incident_radiance(
        &self,
        ray: &mut RayDifferential,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum;

    #[allow(non_snake_case)]
    fn specular_reflect(
        &self,
        ray: &mut RayDifferential,
        intersect: &SurfaceInteraction,
        bsdf: &Bsdf,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum {
        let wo = intersect.wo;
        let bxdf_type = BxDFType::REFLECTION | BxDFType::SPECULAR;

        if let Some(scatter) = bsdf.sample_f(wo, sampler.get_2d(), bxdf_type) {

            if abs_dot(scatter.wi, intersect.shading_n.0) == 0.0 {
                return Spectrum::uniform(0.0);
            }

            let diff = ray.diff.map(|diff| {
                let tex_diff = intersect.tex_diffs;
                let rx_origin = intersect.hit.p + tex_diff.dpdx;
                let ry_origin = intersect.hit.p + tex_diff.dpdy;

                let shading = intersect.shading_geom;
                let dndx = shading.dndu * tex_diff.dudx + shading.dndv * tex_diff.dvdx;
                let dndy = shading.dndu * tex_diff.dudy + shading.dndv * tex_diff.dvdy;

                let dwo_dx = -diff.rx_dir - wo;
                let dwo_dy = -diff.ry_dir - wo;

                let dDN_dx = dwo_dx.dot(intersect.shading_n.0) + wo.dot(dndx.0);
                let dDN_dy = dwo_dy.dot(intersect.shading_n.0) + wo.dot(dndy.0);

                let rx_dir = (scatter.wi - dwo_dx) + 2.0 * wo.dot(intersect.shading_n.0) * dndx.0 + dDN_dx * intersect.shading_n.0;
                let ry_dir = (scatter.wi - dwo_dy) + 2.0 * wo.dot(intersect.shading_n.0) * dndy.0 + dDN_dy * intersect.shading_n.0;

                Differential {
                    rx_origin,
                    rx_dir,
                    ry_origin,
                    ry_dir
                }
            });

            let mut ray_diff = intersect.hit.spawn_ray_with_dfferentials(scatter.wi, diff);
            let li = self.incident_radiance(
                &mut ray_diff,
                scene,
                sampler,
                arena,
                depth + 1
            );
            return scatter.f * li * scatter.wi.dot(intersect.shading_n.0).abs() / scatter.pdf;
        } else {
            return Spectrum::uniform(0.0);
        }
    }

    #[allow(non_snake_case)]
    fn specular_transmit(
        &self,
        ray: &mut RayDifferential,
        intersect: &SurfaceInteraction,
        bsdf: &Bsdf,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        arena: &Bump,
        depth: u16,
    ) -> Spectrum {
        let wo = intersect.wo;
        let bxdf_type = BxDFType::TRANSMISSION | BxDFType::SPECULAR;

        if let Some(scatter) = bsdf.sample_f(wo, sampler.get_2d(), bxdf_type) {

            if abs_dot(scatter.wi, intersect.shading_n.0) == 0.0 {
                return Spectrum::uniform(0.0);
            }

            let diff = ray.diff.map(|diff| {
                let tex_diff = intersect.tex_diffs;
                let rx_origin = intersect.hit.p + tex_diff.dpdx;
                let ry_origin = intersect.hit.p + tex_diff.dpdy;

                let shading = intersect.shading_geom;
                let mut dndx = shading.dndu * tex_diff.dudx + shading.dndv * tex_diff.dvdx;
                let mut dndy = shading.dndu * tex_diff.dudy + shading.dndv * tex_diff.dvdy;
                let mut shading_n = intersect.shading_n;

                // first assume the ray is entering the object and compute relative IOR
                let mut eta = 1.0 / bsdf.eta;
                if wo.dot(intersect.shading_n.0) < 0.0 {
                    eta = bsdf.eta;
                    shading_n = -shading_n;
                    dndx = -dndx;
                    dndy = -dndy;
                }

                let dwo_dx = -diff.rx_dir - wo;
                let dwo_dy = -diff.ry_dir - wo;

                let dDN_dx = dwo_dx.dot(intersect.shading_n.0) + wo.dot(dndx.0);
                let dDN_dy = dwo_dy.dot(intersect.shading_n.0) + wo.dot(dndy.0);

                let mu = eta * wo.dot(shading_n.0) - abs_dot(scatter.wi, shading_n.0);
                let dmu_dx =
                    (eta -
                        (eta * eta * wo.dot(shading_n.0)) / scatter.wi.dot(shading_n.0))
                        * dDN_dx;

                let dmu_dy =
                    (eta -
                        (eta * eta * wo.dot(shading_n.0)) / scatter.wi.dot(shading_n.0))
                        * dDN_dy;

                let rx_dir = scatter.wi - (eta * dwo_dx) + (mu * dndx + dmu_dx * shading_n).0;
                let ry_dir = scatter.wi - (eta * dwo_dy) + (mu * dndy + dmu_dy * shading_n).0;

                Differential {
                    rx_origin,
                    rx_dir,
                    ry_origin,
                    ry_dir
                }
            });

            let mut ray_diff = intersect.hit.spawn_ray_with_dfferentials(scatter.wi, diff);
            let li = self.incident_radiance(
                &mut ray_diff,
                scene,
                sampler,
                arena,
                depth + 1
            );
            return scatter.f * li * scatter.wi.dot(intersect.shading_n.0).abs() / scatter.pdf;
        } else {
            return Spectrum::uniform(0.0);
        }
    }
}

impl<R: IntegratorRadiance> SamplerIntegrator<R> {
    fn tile_id(tile: Bounds2i, sample_bounds: Bounds2i) -> u64 {
        let n_cols = sample_bounds.max.x;
        (tile.min.y * n_cols + tile.min.x) as u64
    }

    fn make_progress_bar(total_size: u64) -> indicatif::ProgressBar {
        let bar = indicatif::ProgressBar::new(total_size);
        bar.set_draw_delta(127);
        bar
    }

    pub fn render_with_pool(&mut self, scene: &Scene, film: &Film<BoxFilter>, sampler: impl Sampler, pool: &rayon::ThreadPool) {
        pool.install(|| self.render(scene, film, sampler))
    }

    pub fn iter_tiles(&self, sample_bounds: Bounds2i, sampler: impl Sampler) -> impl Iterator<Item=(Bounds2i, impl Sampler)> {
        sample_bounds
            .iter_tiles(16)
            .map(move |tile| {
                let tile_id = Self::tile_id(tile, sample_bounds);
                (tile, sampler.clone_with_seed(tile_id))
            })
    }

    pub fn render(&mut self, scene: &Scene, film: &Film<BoxFilter>, mut sampler: impl Sampler) {
        self.radiance.preprocess(scene, &mut sampler);
//        let total_samples = sample_bounds.area() * self.sampler.samples_per_pixel() as i32;
//        let progress = indicatif::ProgressBar::new(total_samples as u64);
        let progress = Self::make_progress_bar(film.sample_bounds().area() as u64);
        self.iter_tiles(film.sample_bounds(), sampler)
            .for_each(|(tile, mut tile_sampler)| {
                self.render_tile(scene, film, tile_sampler, tile, &progress)
            });
       progress.finish();
    }

    pub fn render_parallel(&mut self, scene: &Scene, film: &Film<BoxFilter>, mut sampler: impl Sampler) {
        self.radiance.preprocess(scene, &mut sampler);
        let tiles: Vec<_> = self.iter_tiles(film.sample_bounds(), sampler).collect();
        let progress = Self::make_progress_bar(film.sample_bounds().area() as u64);
        let prog_ref = &progress; // because of move
        tiles.into_par_iter().for_each(move |(tile, mut tile_sampler)| {
            self.render_tile(scene, film, tile_sampler, tile, &prog_ref);
        });
        progress.finish()
    }

    fn render_tile(&self,
                   scene: &Scene,
                   film: &Film<BoxFilter>,
                   mut tile_sampler: impl Sampler,
                   tile: Bounds2i,
                   progress: &indicatif::ProgressBar
    ) {
        let mut arena = Bump::new();

        let mut film_tile = film.get_film_tile(tile);

        for pixel in tile.iter_points() {
            tile_sampler.start_pixel(pixel.into());

            while tile_sampler.start_next_sample() {
                let camera_sample = tile_sampler.get_camera_sample(pixel.into());

                let (ray_weight, mut ray_differential) =
                    self.camera.generate_ray_differential(camera_sample);

                ray_differential.scale_differentials(
                    1.0 / (tile_sampler.samples_per_pixel() as Float).sqrt(),
                );

                let mut radiance = Spectrum::uniform(0.0);

                if ray_weight > 0.0 {
                    radiance = self.radiance.incident_radiance(
                        &mut ray_differential,
                        scene,
                        &mut tile_sampler,
                        &arena,
                        0,
                    );

                    check_radiance(&radiance, pixel);
                }

                film.add_sample_to_tile(
                    &mut film_tile,
                    camera_sample.p_film,
                    radiance,
                    ray_weight,
                );

                arena.reset();
            }

            progress.inc(1);
        }

        film.merge_film_tile(film_tile);
    }

}

fn check_radiance(l: &Spectrum, pixel: (i32, i32)) {
    assert!(!l.has_nans(), "NaN radiance value for pixel {:?}: {:?}", pixel, l);
}

pub fn uniform_sample_one_light(
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

pub fn estimate_direct(
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

    // Sample light source with multiple importance sampling
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

    // Sample BSDF with multiple importance sampling.
    // If the light source involves a delta distribution then the BSDF cannot be sampled since there
    // is a zero probability that it will sample a direction that receives light from the source
    if !light.flags().is_delta_light() {
        let scatter = bsdf.sample_f(intersect.wo, u_scattering, bsdf_flags);
        if let Some(scatter) = scatter {
            let f = scatter.f * abs_dot(scatter.wi, intersect.shading_n.0);
            let sampled_specular = scatter.sampled_type.contains(BxDFType::SPECULAR);

            if f.is_black() {
                return radiance;
            }

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
