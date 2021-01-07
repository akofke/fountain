use crate::{Float, Transform, Point2f, Vec3f, distance_sq, abs_dot};
use crate::geometry::Ray;
use crate::geometry::bounds::Bounds3f;
use crate::interaction::{SurfaceInteraction, SurfaceHit};

pub mod sphere;
pub mod triangle;
pub mod loop_subdiv;

pub trait Shape: Sync + Send {
    fn object_bound(&self) -> Bounds3f;

    fn world_bound(&self) -> Bounds3f {
        self.object_to_world().transform(self.object_bound())
    }

    fn object_to_world(&self) -> &Transform;

    fn world_to_object(&self) -> &Transform;

    fn transform_swaps_handedness(&self) -> bool {
        self.object_to_world().swaps_handedness()
    }

    fn reverse_orientation(&self) -> bool;

    fn flip_normals(&self) -> bool {
        self.reverse_orientation() ^ self.transform_swaps_handedness()
    }

    fn area(&self) -> Float;

    fn intersect(&self, ray: &Ray) -> Option<(Float, SurfaceInteraction)>;

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    /// Choose a point on the surface of the shape using a sampling distribution with respect to
    /// surface area.
    fn sample(&self, u: Point2f) -> SurfaceHit;

    fn pdf(&self, _hit: &SurfaceHit) -> Float {
        1.0 / self.area()
    }

    /// Samples the surface of the shape, taking into account the point from which the surface is
    /// being integrated over. Uses a density with respect to solid angle from the reference point.
    ///
    /// The default implementation ignores the reference point and calls `sample`.
    fn sample_from_ref(&self, _reference: &SurfaceHit, u: Point2f) -> SurfaceHit {
        self.sample(u)
    }

    fn pdf_from_ref(&self, reference: &SurfaceHit, wi: Vec3f) -> Float {
        let ray = reference.spawn_ray(wi);

        if let Some((_, isect_light)) = self.intersect(&ray) {
            // convert from a density with respect to area to a density with respect
            // to solid angle
            distance_sq(reference.p, isect_light.hit.p) /
                (abs_dot(isect_light.hit.n.0, -wi) * self.area())
        } else {
            0.0
        }
    }

}
