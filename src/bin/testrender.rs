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
use raytracer::light::point::PointLight;
use raytracer::spectrum::Spectrum;
use raytracer::light::Light;
use raytracer::light::distant::DistantLight;
use raytracer::point3f;
use cgmath::vec3;
use raytracer::material::mirror::MirrorMaterial;
use raytracer::texture::ConstantTexture;
use raytracer::material::glass::GlassMaterial;

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

    let o2w = Transform::translate((2.0, 1.0, 0.0).into());
    let w2o = o2w.inverse();
    let sphere2 = Sphere::new(
        &o2w,
        &w2o,
        false,
        1.0,
        -1.0,
        1.0,
        360.0
    );

    let o2w = Transform::translate((0.0, -2.0, 0.0).into());
    let w2o = o2w.inverse();
    let sphere3 = Sphere::new(
        &o2w,
        &w2o,
        false,
        1.0,
        -1.0,
        1.0,
        360.0
    );

    let o2w = Transform::translate((0.0, 0.0, -21.0).into());
    let w2o = o2w.inverse();
    let ground_sphere = Sphere::whole(
        &o2w, &w2o, 20.0
    );

    let blue = Arc::new(MatteMaterial::constant([0.2, 0.2, 0.7].into()));
    let red = Arc::new(MatteMaterial::constant([0.7, 0.2, 0.2].into()));
    let green = Arc::new(MatteMaterial::constant([0.2, 0.7, 0.2].into()));
    let mirror = Arc::new(MirrorMaterial::new(Arc::new(ConstantTexture(Spectrum::new(0.9)))));
    let glass = Arc::new(GlassMaterial::constant(Spectrum::new(1.0), Spectrum::new(1.0), 1.1));

    let prim = GeometricPrimitive {
        shape: sphere,
        material: Some(glass.clone())
    };

    let prim2 = GeometricPrimitive {
        shape: sphere2,
        material: Some(green.clone())
    };

    let prim3 = GeometricPrimitive {
        shape: sphere3,
        material: Some(blue.clone())
    };

    let ground_prim = GeometricPrimitive {
        shape: ground_sphere,
        material: Some(red.clone()),
    };

    let prims: Vec<&dyn Primitive> = vec![
        &prim,
        &ground_prim,
        &prim2,
        &prim3,
    ];
    let bvh = BVH::build(prims);

    let mut light = PointLight::new(Transform::translate((0.0, 0.0, 3.0).into()), Spectrum::new(10.0));
    let mut dist_light = DistantLight::new(Spectrum::new(1.0), vec3(3.0, 3.0, 3.0));
    let lights: Vec<&mut dyn Light> = vec![
        &mut dist_light,
        &mut light,
    ];
//    let lights: Vec<&mut dyn Light> = vec![&mut light];
    let scene = Scene::new(bvh, lights);

    let resolution = Point2i::new(512, 512);

//    let camera_pos = Transform::translate((0.0, 0.0, 10000.0).into());
    let camera_tf = Transform::camera_look_at(
        (0.0, 4.0, 4.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 0.0, 1.0).into()
    );
    let camera = PerspectiveCamera::new(
        camera_tf,
        resolution,
        Bounds2::whole_screen(),
        (0.0, 1.0),
        0.0,
        1.0e6,
        60.0
    );
    let camera = Box::new(camera);
    let sampler = Box::new(RandomSampler::new_with_seed(8, 1));
    let radiance = WhittedIntegrator { max_depth: 4 };
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

    let pool = ThreadPoolBuilder::new()
//        .num_threads(1)
        .build().unwrap();
    integrator.render_with_pool(&scene, &film, &pool);

    let img = film.into_image_buffer();
    let mut file = File::create("testrender.hdr").unwrap();
    let encoder = image::hdr::HDREncoder::new(file);
    let pixels: Vec<_> = img.pixels().map(|p| *p).collect();
    encoder.encode(pixels.as_slice(), img.width() as usize, img.height() as usize).unwrap();
}