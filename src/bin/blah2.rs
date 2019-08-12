use rand::distributions::{Uniform, UnitSphereSurface};
use raytracer::{Transform, SurfaceInteraction, Ray, Vec3f};
use cgmath::Vector3;
use raytracer::bvh::BVH;
use raytracer::primitive::{GeometricPrimitive, Primitive};
use raytracer::shapes::sphere::Sphere;
use rand::prelude::*;

fn main() {
    test_bvh_intersect_many_nodes()
}

//noinspection DuplicatedCode
fn test_bvh_intersect_many_nodes() {
    let mut rng = StdRng::from_seed([3; 32]);
    let distr = Uniform::new_inclusive(-10.0, 10.0);
    let tfs: Vec<(Transform, Transform)> = (0..5)
        .map(|_| {
            let v = Vec3f::new(rng.sample(distr), rng.sample(distr), rng.sample(distr));
            let o2w = Transform::translate(v);
            (o2w, o2w.inverse())
        })
        .collect();

    let prims: Vec<GeometricPrimitive<Sphere>> = tfs.iter()
        .map(|(o2w, w2o)| {
            let sphere = Sphere::whole(o2w, w2o, rng.gen_range(0.5, 5.0));
            GeometricPrimitive { shape: sphere, material: None }
        })
        .collect();

    let mut prim_refs: Vec<&dyn Primitive> = vec![];
    for p in &prims {
        prim_refs.push(p);
    }

    let bvh = BVH::build(prim_refs.clone());

    let mut sphere_surf = UnitSphereSurface::new();
    for i in 0..100 {
        let dir = sphere_surf.sample(&mut rng);
        let dir: Vec3f = Vector3::from(dir).cast().unwrap();
        let mut ray = Ray::new((0.0, 0.0, 0.0).into(), dir);

        let mut bvh_ray = ray.clone();
        let bvh_isect_test = bvh.intersect_test(&bvh_ray);
        let bvh_isect = bvh.intersect(&mut bvh_ray);

        let expected_test = intersect_test_list(&ray, &prim_refs);
        let expected_isect = intersect_list(&mut ray, &prim_refs);

        assert_eq!(expected_test, expected_isect.is_some(), "Iteration {}", i);
        assert_eq!(bvh_isect_test, bvh_isect.is_some(), "Iteration {}", i);
        assert_eq!(bvh_isect.map(|i| i.hit), expected_isect.map(|i| i.hit), "Iteration {}", i);
        assert_eq!(bvh_isect_test, expected_test, "Iteration {}", i);
    }
}

fn intersect_test_list(ray: &Ray, prims: &[&dyn Primitive]) -> bool {
    prims.iter().any(|prim| {
        prim.intersect_test(ray)
    })
}

fn intersect_list<'p>(ray: &mut Ray, prims: &'p [&dyn Primitive]) -> Option<SurfaceInteraction<'p>> {
    let mut isect = None;
    for prim in prims {
        isect = prim.intersect(ray).or(isect);
    }
    isect
}
