use criterion::{Criterion, criterion_main, criterion_group, Throughput};
use raytracer::loaders::constructors::make_triangle_mesh_from_ply;
use std::path::PathBuf;
use raytracer::loaders::pbrt::{PbrtHeader, PbrtSceneBuilder};
use raytracer::sampler::random::RandomSampler;
use raytracer::Point2i;
use rand::Rng;
use raytracer::sampler::Sampler;

fn bench(c: &mut Criterion) {
    let path = PathBuf::from("/Users/alex/scenes/pbrt-v3-scenes/ganesha/ganesha.pbrt");
    let base_path = path.parent().unwrap().to_path_buf();
    let parsed = pbrt_parser::PbrtParser::parse_with_includes(&path).unwrap();
    let mut header = PbrtHeader::new();
    for stmt in parsed.header {
        header.exec_stmt(stmt).unwrap();
    }

    let mut scene_builder = PbrtSceneBuilder::new(base_path);
    for stmt in parsed.world {
        scene_builder.exec_stmt(stmt).unwrap();
    }

    let scene = scene_builder.create_scene();
    let camera = header.make_camera().unwrap();

    let mut sampler = RandomSampler::new_with_seed(512, 1);
    let mut rng = rand::thread_rng();
    let full_res = header.make_film().unwrap().full_resolution;

    let mut group = c.benchmark_group("Ganesha");
    group.throughput(Throughput::Elements(1));
    group.bench_function("scene intersect", |b| {
        b.iter(|| {
            let pixel = Point2i::new(rng.gen_range(0, full_res.x), rng.gen_range(0, full_res.y));
            sampler.start_pixel(pixel);
            let camera_sample = sampler.get_camera_sample(pixel);
            let (wt, mut ray) = camera.generate_ray(camera_sample);
            scene.intersect(&mut ray);
        })
    });
    group.finish();


}

criterion_group!(benches, bench);
criterion_main!(benches);

