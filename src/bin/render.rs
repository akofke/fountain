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

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.0.1")]
struct Opts {
    scene_file: PathBuf,

    #[clap(short = "t", long = "threads", default_value = "0")]
    threads: usize,
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    let base_path = opts.scene_file.parent().unwrap().to_path_buf();

    let parsed = pbrt_parser::PbrtParser::parse_with_includes(&opts.scene_file)?;

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
    if opts.threads != 1 {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(opts.threads)
            .build()?;
        integrator.render_with_pool(&scene, &film, sampler, &pool);
    } else {
        integrator.render(&scene, &film, sampler);
    }

    let (img, (w, h)) = film.into_spectrum_buffer();
    let mut file = File::create(filename)?;
    write_exr(&mut file, img, (w, h))?;
    Ok(())
}