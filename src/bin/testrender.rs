use raytracer::integrator::{SamplerIntegrator, Integrator};
use raytracer::sampler::random::RandomSampler;
use raytracer::camera::PerspectiveCamera;
use raytracer::{Transform, Point2i, Bounds2};
use raytracer::integrator::whitted::WhittedIntegrator;
use raytracer::scene::Scene;
use raytracer::bvh::BVH;
use raytracer::shapes::sphere::Sphere;
use raytracer::primitive::{GeometricPrimitive, Primitive};
use raytracer::material::matte::MatteMaterial;
use std::sync::Arc;
use raytracer::film::Film;
use raytracer::filter::BoxFilter;
use std::fs::File;
use rayon::ThreadPoolBuilder;

pub fn main() {

    let o2w = Transform::translate((0.0, 0.0, 0.0).into());
    let w2o = o2w.inverse();
    let sphere = Sphere::new(
        &o2w,
        &w2o,
        false,
        1.0,
        -1.0,
        1.0,
        360.0
    );

    let mat = MatteMaterial::constant([0.0, 1.0, 0.0].into());

    let prim = GeometricPrimitive {
        shape: sphere,
        material: Some(Arc::new(mat))
    };

    let prims: Vec<&dyn Primitive> = vec![&prim];
    let bvh = BVH::build(prims);
    let scene = Scene { primitives_aggregate: bvh };

    let resolution = Point2i::new(256, 256);

//    let camera_pos = Transform::translate((0.0, 0.0, 10000.0).into());
    let camera_tf = Transform::camera_look_at(
        (3.0, 3.0, 3.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 0.0, 1.0).into()
    );
    let camera = PerspectiveCamera::new(
        camera_tf,
        resolution,
        Bounds2::unit(),
        (0.0, 1.0),
        0.0,
        1.0e6,
        39.0
    );
    let camera = Box::new(camera);
    let sampler = Box::new(RandomSampler::new_with_seed(1, 1));
    let radiance = WhittedIntegrator { max_depth: 1 };
    let mut integrator = SamplerIntegrator {
        sampler,
        camera,
        radiance
    };

    let film = Film::new(
        resolution,
        Bounds2::unit(),
        BoxFilter::default(),
        1.0
    );

    let pool = ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    integrator.render_with_pool(&scene, &film, &pool);

    let img = film.into_image_buffer();
    let mut file = File::create("testrender.hdr").unwrap();
    let encoder = image::hdr::HDREncoder::new(file);
    let pixels: Vec<_> = img.pixels().map(|p| *p).collect();
    encoder.encode(pixels.as_slice(), img.width() as usize, img.height() as usize).unwrap();
}