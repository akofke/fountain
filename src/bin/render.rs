use std::error::Error;
use std::env::args;
use raytracer::loaders::pbrt::{PbrtHeader, PbrtSceneBuilder};
use raytracer::integrator::SamplerIntegrator;
use raytracer::integrator::direct_lighting::{DirectLightingIntegrator, LightStrategy};
use std::fs::File;
use raytracer::imageio::exr::write_exr;
use raytracer::integrator::whitted::WhittedIntegrator;
use raytracer::integrator::path::PathIntegrator;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let path = args().nth(1).unwrap();

    let path = PathBuf::from(path);
    let base_path = path.parent().unwrap().to_path_buf();
    let parsed = pbrt_parser::PbrtParser::parse_with_includes(&path)?;

    let mut header = PbrtHeader::new();
    for stmt in parsed.header {
        header.exec_stmt(stmt)?;
    }
    let filename = header.film_params.get_one("filename").unwrap_or("render.exr".to_string());
    assert!(filename.contains(".exr"));

    let mut scene_builder = PbrtSceneBuilder::new(base_path);
    for stmt in parsed.world {
        scene_builder.exec_stmt(stmt)?;
    }

    let scene = scene_builder.create_scene();

    let camera = header.make_camera()?;
    let sampler = header.make_sampler()?;
    let film = header.make_film()?;

    let mut integrator = SamplerIntegrator {
        camera,
        // radiance: WhittedIntegrator {
        //     max_depth: 4
        // }
        // radiance: DirectLightingIntegrator {
        //     strategy: LightStrategy::UniformSampleOne,
        //     max_depth: 4,
        //     n_light_samples: vec![],
        // }
        radiance: PathIntegrator::new(5, 1.0)
    };

    dbg!(&scene);
    let parallel = true;
    if parallel {
        integrator.render_parallel(&scene, &film, sampler);
    } else {
        integrator.render(&scene, &film, sampler);
    }

    let (img, (w, h)) = film.into_spectrum_buffer();
    let mut file = File::create(filename)?;
    write_exr(&mut file, img, (w, h))?;
    Ok(())
}