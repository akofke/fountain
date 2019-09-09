#![allow(clippy::all)]
#![allow(unused_imports, unused_variables)]
use raytracer::integrator::{SamplerIntegrator, Integrator};
use raytracer::sampler::random::RandomSampler;
use raytracer::camera::PerspectiveCamera;
use raytracer::{Transform, Point2i, Bounds2, Point3f, Vec3f, Normal3, Point2f, Vec2f};
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
use cgmath::{vec3, Deg, Angle};
use raytracer::material::mirror::MirrorMaterial;
use raytracer::texture::ConstantTexture;
use raytracer::material::glass::GlassMaterial;
use raytracer::texture::uv::UVTexture;
use raytracer::texture::mapping::UVMapping;
use raytracer::texture::checkerboard::Checkerboard2DTexture;
use raytracer::shapes::triangle::TriangleMesh;
use std::path::Path;
use tobj::load_obj;
use std::error::Error;

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

    let o2w = Transform::translate((0.0, 0.0, -31.0).into());
    let w2o = o2w.inverse();
    let ground_sphere = Sphere::whole(
        &o2w, &w2o, 20.0
    );

    let points = vec![
        Point3f::new(0.0, 0.0, 0.0),
        Point3f::new(0.0, 0.0, 1.0),
        Point3f::new(1.0, 0.0, 0.0),
        Point3f::new(1.0, 0.0, 1.0),
    ];
    let mesh = TriangleMesh::new(
        Transform::identity(),
        vec![0, 1, 2, 2, 1, 3],
        points,
        None,
        None,
        None,
        false
    );

    let meshes = mesh_from_obj("bunny.obj");

    let blue = Arc::new(MatteMaterial::constant([0.2, 0.2, 0.7].into()));
    let red = Arc::new(MatteMaterial::constant([0.7, 0.2, 0.2].into()));
    let green = Arc::new(MatteMaterial::constant([0.2, 0.7, 0.2].into()));
    let uv = Arc::new(MatteMaterial::new(Arc::new(UVTexture::new(UVMapping::default()))));
    let check = Arc::new(MatteMaterial::new(Arc::new(
        Checkerboard2DTexture::new(ConstantTexture(Spectrum::new(0.0)), ConstantTexture(Spectrum::new(1.0)), UVMapping::new(100.0, 100.0, 0.0, 0.0))
    )));
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
        material: Some(uv.clone())
    };

    let ground_prim = GeometricPrimitive {
        shape: ground_sphere,
        material: Some(check.clone()),
    };


    let tri_prims: Vec<Box<dyn Primitive>> = meshes.iter().flat_map(|mesh| {
        mesh.iter_triangles()
            .map(|tri| {
                Box::new(GeometricPrimitive {
                    shape: tri,
                    material: Some(uv.clone())
                }) as Box<dyn Primitive>
            })
    }).collect();

    let mut prims: Vec<&dyn Primitive> = vec![
//        &prim,
//        &ground_prim,
//        &prim2,
//        &prim3,
    ];
    prims.extend(tri_prims.iter().map(|b| b.as_ref()));
    let bvh = BVH::build(prims);
    dbg!(bvh.bounds);

    let mut light = PointLight::new(Transform::translate((0.0, 0.0, 3.0).into()), Spectrum::new(10.0));
    let mut dist_light = DistantLight::new(Spectrum::new(10.5), vec3(3.0, 3.0, 3.0));
    let lights: Vec<&mut dyn Light> = vec![
        &mut dist_light,
        &mut light,
    ];
//    let lights: Vec<&mut dyn Light> = vec![&mut light];
    let scene = Scene::new(bvh, lights);

    let resolution = Point2i::new(1024, 1024);

//    let camera_pos = Transform::translate((0.0, 0.0, 10000.0).into());
    let camera_tf = Transform::camera_look_at(
        (0.0, 10.0, 10.0).into(),
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
        .num_threads(1)
        .build().unwrap();
    integrator.render_with_pool(&scene, &film, &pool);

    let img = film.into_image_buffer();
    let file = File::create("testrender.hdr").unwrap();
    let encoder = image::hdr::HDREncoder::new(file);
    let pixels: Vec<_> = img.pixels().map(|p| *p).collect();
    encoder.encode(pixels.as_slice(), img.width() as usize, img.height() as usize).unwrap();
}

fn mesh_from_obj(path: impl AsRef<Path>) -> Vec<TriangleMesh> {
    let (models, materials) = match tobj::load_obj(path.as_ref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e.description());
            panic!();
        }
    };
    models.into_iter().map(|model| {
//        dbg!(&model.mesh.positions);
        let vertices = model.mesh.positions
            .chunks_exact(3)
            .map(|v| Point3f::new(v[0], v[1], v[2]))
            .collect();
//        dbg!(&model.mesh.indices);
        let normals: Vec<Normal3> = model.mesh.normals
            .chunks_exact(3)
            .map(|v| Normal3::new(v[0], v[1], v[2]))
            .collect();
        let normals = if normals.is_empty() { None } else { Some(normals) };
        let tex_coords: Vec<Point2f> = model.mesh.texcoords
            .chunks_exact(2)
            .map(|v| Point2f::new(v[0], v[1]))
            .collect();
        let tex_coords = if tex_coords.is_empty() { None } else { Some(tex_coords) };

        let tf = Transform::identity();
        let tf = Transform::scale(4.0, 4.0, 4.0) * tf;
        let tf = Transform::rotate_z(Deg::turn_div_2()) * tf;
        let tf = Transform::rotate_x(Deg(-45.0)) * tf;
//        let tf = Transform::scale(1.0 / 45.0, 1.0/45.0, 1.0/45.0) * tf;
//        let tf = Transform::translate((0.0, 0.0, 2.0).into()) * tf;
        let mesh = TriangleMesh::new(
            tf,
            model.mesh.indices,
            vertices,
            normals,
            None,
            tex_coords,
            false
        );
        mesh
    }).collect()
}