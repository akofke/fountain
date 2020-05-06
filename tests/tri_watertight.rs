/*!
This is a general integration test that exercises BVH, triangle intersect, and attempts to verify
the watertightness of the triangle intersect
*/

use std::path::{Path, PathBuf};
use raytracer::scene::Scene;
use raytracer::loaders::constructors::make_triangle_mesh_from_ply;
use raytracer::loaders::{ParamSet, Context};
use std::sync::Arc;
use raytracer::primitive::{GeometricPrimitive, Primitive};
use raytracer::bvh::BVH;
use rand::thread_rng;
use rand::distributions::{UnitSphereSurface, Distribution};
use raytracer::{Vec3f, Float, Ray, Point3f, Transform};
use cgmath::EuclideanSpace;

#[test]
fn test_rounded_cube() -> anyhow::Result<()> {
    let scene = load_mesh("data/rounded_cube.ply")?;
    test_mesh(&scene);
    Ok(())
}

fn test_mesh(scene: &Scene) {
    let mut rng = thread_rng();
    UnitSphereSurface::new().sample_iter(&mut rng)
        .take(100_000)
        .for_each(|[x, y, z]| {
            let dir = Vec3f::new(x as Float, y as Float, z as Float);
            let mut ray = Ray::new(Point3f::origin(), dir);

            assert!(scene.intersect_test(&ray));

            let _isect = scene.intersect(&mut ray).expect("Did not intersect");
        })
}

fn load_mesh(path: impl AsRef<Path>) -> anyhow::Result<Scene> {
    let mut params = ParamSet::default();
    params
        .with(
            "filename",
            Path::new(env!("CARGO_MANIFEST_DIR")).join(path.as_ref()).to_str().unwrap().to_string()
        )
        .with("object_to_world", Transform::identity())
        .with("reverse_orientation", false);
    let ctx = Context::new(Path::new(env!("CARGO_MANIFEST_DIR")).into());
    let mesh = Arc::new(make_triangle_mesh_from_ply(params, &ctx).unwrap());
    let prims = mesh.clone().iter_triangles()
        .map(|tri| {
            let tri = Arc::new(tri);
            let prim = GeometricPrimitive {
                shape: tri,
                material: None,
                light: None
            };
            Box::new(prim) as Box<dyn Primitive>
        })
        .collect();
    let bvh = BVH::build(prims);
    let scene = Scene::new(
        bvh,
        vec![],
        vec![mesh],
    );
    Ok(scene)
}

