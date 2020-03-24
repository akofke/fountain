use std::error::Error;
use std::env::args;
use raytracer::loaders::pbrt::{PbrtHeader, PbrtSceneBuilder};
use raytracer::integrator::SamplerIntegrator;
use raytracer::integrator::direct_lighting::{DirectLightingIntegrator, LightStrategy};
use std::fs::File;
use raytracer::imageio::exr::write_exr;

fn main() -> Result<(), Box<dyn Error>> {
    let path = args().nth(1).unwrap();

    let parsed = pbrt_parser::PbrtParser::parse_with_includes(path)?;

    let mut header = PbrtHeader::new();
    for stmt in parsed.header {
        header.exec_stmt(stmt)?;
    }

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
        radiance: DirectLightingIntegrator {
            strategy: LightStrategy::UniformSampleOne,
            max_depth: 4,
            n_light_samples: vec![],
        }
    };

    dbg!(&scene);
    integrator.render_parallel(&scene, &film, sampler);

    let (img, (w, h)) = film.into_spectrum_buffer();
    let mut file = File::create("testrender5.exr")?;
    write_exr(&mut file, img, (w, h))?;
    Ok(())
}