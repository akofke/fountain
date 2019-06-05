use crate::Vec3;
use std::f32;

/// Axis-aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3
}

pub trait Bounded {
    fn aabb(&self) -> Aabb;
}

pub enum Axis { X = 0, Y, Z }

impl Aabb {
    pub fn with_bounds(min: Vec3, max: Vec3) -> Self {
        Self {min, max}
    }

    pub fn empty() -> Self {
        Self::with_bounds(Vec3::repeat(f32::INFINITY), Vec3::repeat(f32::NEG_INFINITY))
    }

    pub fn join(&self, other: &Aabb) -> Self {
        Self::with_bounds(
            Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            )

        )
    }

    pub fn join_point(&self, point: &Vec3) -> Self {
        Self::with_bounds(
            Vec3::new(
                self.min.x.min(point.x),
                self.min.y.min(point.y),
                self.min.z.min(point.z),
            ),

            Vec3::new(
                self.max.x.max(point.x),
                self.max.y.max(point.y),
                self.max.z.max(point.z),
            )
        )
    }

    /// Produces a normalized Aabb whose coordinates all lie within [0, 1) with
    /// the given world Aabb used as a scale.
    ///
    /// ```
    /// use raytracer::math::Vec3;
    /// use raytracer::aabb::Aabb;
    ///
    /// let world = Aabb::with_bounds(Vec3::new(-50.0, -50.0, -50.0), Vec3::new(50.0, 50.0, 50.0));
    /// let bb = Aabb::with_bounds(Vec3::new(-20.0, 0.0, 10.0), Vec3::new(-10.0, 10.0, 30.0));
    /// let scaled = bb.normalized_by(&world);
    /// assert_eq!(scaled.min, Vec3::new(0.3, 0.5, 0.6));
    /// assert_eq!(scaled.max, Vec3::new(0.4, 0.6, 0.8));
    /// ```
    pub fn normalized_by(&self, world: &Aabb) -> Self {
        let size = world.size();
        let norm = Self::with_bounds((self.min - world.min).component_div(&size), (self.max - world.min).component_div(&size));

//        debug_assert!(norm.min.x >= 0.0 && norm.min.x < 1.0);
//        debug_assert!(norm.min.y >= 0.0 && norm.min.y < 1.0);
//        debug_assert!(norm.min.z >= 0.0 && norm.min.z < 1.0);
//        debug_assert!(norm.max.x >= 0.0 && norm.max.x < 1.0);
//        debug_assert!(norm.max.y >= 0.0 && norm.max.y < 1.0);
//        debug_assert!(norm.max.z >= 0.0 && norm.max.z < 1.0);

        norm
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn centroid(&self) -> Vec3 {
        self.min + (self.size() / 2.0)
    }

    pub fn diagonal(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn maximum_extent(&self) -> Axis {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            Axis::X
        } else if d.y > d.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn is_point(&self) -> bool {
        self.max == self.min
    }
}