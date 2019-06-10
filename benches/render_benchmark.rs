use criterion::{Criterion, criterion_group, criterion_main, BatchSize};
use raytracer::renderer_old::Renderer;
use raytracer::scene::Scene;
use raytracer::camera::Camera;
use raytracer::camera::Lens;
use raytracer::geom::Sphere;
use raytracer::material::*;
use raytracer::{Vec3f, v3};
use raytracer::cover_example_scene;
use std::time::Duration;

fn bench(c: &mut Criterion) {
    c.bench_function("cover_scene_time_per_pixel", |b| {
        let w = 500;
        let h = 250;
        let r = get_renderer(w, h);
        let mut pixels = r.iter_pixels(w, h);
        b.iter(|| {
            pixels.next()
        })
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = bench
}
criterion_main!(benches);

fn get_renderer(w: usize, h: usize) -> Renderer {
    let aspect = w as f32 / h as f32;
    let (s, c) = cover_example_scene(aspect);
    Renderer::new(s, c)
}