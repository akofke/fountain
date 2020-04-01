use raytracer::loaders::pbrt::{PbrtHeader, PbrtSceneBuilder};
use raytracer::integrator::{SamplerIntegrator, IntegratorRadiance};
use raytracer::integrator::path::PathIntegrator;

use approx::assert_abs_diff_eq;
use std::path::Path;
use raytracer::spectrum::Spectrum;
use raytracer::integrator::direct_lighting::{DirectLightingIntegrator, LightStrategy};

#[test]
fn furnace_test_path() -> anyhow::Result<()> {
    let (img, (w, h)) =
        do_render(PathIntegrator::new(10, 1.0), "testscenes/furnace_empty.pbrt")?;

    let expected = 1.0 / (1.0 - 0.5);
    // TODO: could use an actual statistical test (as in Nori)
    for s in img {
        for comp in s.into_array().iter() {
            // Russian roulette causes some variance
            assert_abs_diff_eq!(*comp, expected, epsilon = 0.1);
        }
    }

    Ok(())
}

#[test]
fn furnace_test_path_no_rr() -> anyhow::Result<()> {
    let (img, (w, h)) =
        do_render(PathIntegrator::new(10, 0.0), "testscenes/furnace_empty.pbrt")?;

    let expected = 1.0 / (1.0 - 0.5);
    for s in img {
        for comp in s.into_array().iter() {
            // No Russian roulette should mean the same value for every sample
            assert_abs_diff_eq!(*comp, expected, epsilon = 0.001);
        }
    }

    Ok(())
}

#[test]
fn furnace_test_directlighting() -> anyhow::Result<()> {
    let (img, (w, h)) =
        do_render(DirectLightingIntegrator {
            strategy: LightStrategy::UniformSampleOne,
            max_depth: 3,
            n_light_samples: vec![]
        }, "testscenes/furnace_empty.pbrt")?;

    let expected = 1.0 + 0.5;
    for s in img {
        for comp in s.into_array().iter() {
            assert_abs_diff_eq!(*comp, expected, epsilon = 0.00001);
        }
    }

    Ok(())
}

fn do_render(integrator: impl IntegratorRadiance, fname: impl AsRef<Path>) -> anyhow::Result<(Vec<Spectrum>, (u32, u32))> {

    let parsed = pbrt_parser::PbrtParser::parse_with_includes(fname)?;

    let mut header = PbrtHeader::new();
    for stmt in parsed.header {
        header.exec_stmt(stmt)?;
    }
    let filename = header.film_params.get_one("filename").unwrap_or("render.exr".to_string());
    assert!(filename.contains(".exr"));

    let mut scene_builder = PbrtSceneBuilder::new();
    for stmt in parsed.world {
        scene_builder.exec_stmt(stmt)?;
    }

    let scene = scene_builder.create_scene();

    let camera = header.make_camera()?;
    let sampler = header.make_sampler()?;
    let film = header.make_film()?;

    let mut integrator = SamplerIntegrator {
        camera,
        radiance: integrator
    };

    let parallel = true;
    if parallel {
        integrator.render_parallel(&scene, &film, sampler);
    } else {
        integrator.render(&scene, &film, sampler);
    }

    Ok(film.into_spectrum_buffer())
}