use std::error::Error;
use std::env::args;
use raytracer::loaders::pbrt::{PbrtHeader, PbrtSceneBuilder};
use raytracer::integrator::SamplerIntegrator;
use raytracer::integrator::direct_lighting::{DirectLightingIntegrator, LightStrategy};
use std::fs::File;

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

    let img = film.into_image_buffer();
    let file = File::create("testrender4.hdr")?;
    let encoder = image::hdr::HDREncoder::new(file);
    let pixels: Vec<_> = img.pixels().map(|p| *p).collect();
    encoder.encode(pixels.as_slice(), img.width() as usize, img.height() as usize)?;
    Ok(())
}