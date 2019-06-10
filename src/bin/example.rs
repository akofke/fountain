use raytracer::geom::Sphere;
use raytracer::material::*;
use raytracer::camera::*;
use raytracer::image::write_ppm_ascii;
use raytracer::*;
use raytracer::scene::Scene;
use raytracer::renderer_old::Renderer;

fn main() {
    let width = 1000;
    let height = 500;
    let aspect = width as f32 / height as f32;
    let spheres: Vec<Sphere> = vec![
        Sphere::fixed(Vec3::new(0.0, 0.0, -2.0), 0.5, Box::new(Lambertian {albedo: Vec3::new(0.8, 0.8, 0.0)})),
        Sphere::fixed(Vec3::new(1.0, 0.0, -2.0), 0.5, Box::new(Dielectric {refractive_index: 1.5})),
        Sphere::new(Vec3::new(1.0, 0.5, -4.0), 1.0, Box::new(Lambertian {albedo: Vec3::new(0.3, 0.8, 0.3)}), Some(v3!(0.5, 0.5, -0.5))),
        Sphere::fixed(Vec3::new(-1.0, 0.0, -2.0), 0.5, Box::new(Metal {albedo: Vec3::new(0.8, 0.6, 0.2), fuzz: 0.3})),
        Sphere::fixed(Vec3::new(0.0, -100.5, -1.0), 100.0, Box::new(Lambertian {albedo: Vec3::new(0.3, 0.3, 0.8)})) // horizon-ish
    ];

    let lookfrom = Vec3::new(3., 3., 2.);
    let lookat = Vec3::new(0., 0., -2.);

    let lens = Lens {focus_dist: (lookfrom - lookat).norm(), aperture: 0.5};

    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0., 1., 0.),
        60.0f32.to_radians(),
        aspect,
        Some(lens),
        Some((0.0, 1.0))
    );
//    let camera = Camera::with_aspect(aspect);
//    let framebuf = render(width, height, spheres, &camera);
    let scene = Scene::new(spheres);
    let framebuf = Renderer::new(scene, camera).render_parallel(width, height);

    write_ppm_ascii(width, height, &framebuf, "test2.ppm").expect("Failed to write file");
}