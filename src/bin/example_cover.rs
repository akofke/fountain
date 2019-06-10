use raytracer::cover_example_scene;
use raytracer::renderer_old::Renderer;
use raytracer::image::write_ppm;

fn main() {
    let w = 1000;
    let h = 500;
    let (scene, camera) = cover_example_scene(w as f32 / h as f32);
    let r = Renderer::new(scene, camera);
    let framebuf = r.render_parallel(w, h);
    write_ppm(w, h, &framebuf, "cover_example.ppm").expect("Error writing file");
}